use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use log::{debug, error, info, warn};
use tokio::sync::mpsc;

use crate::tmdb::{MediaItem, SeasonDetail, TmdbClient, TvDetail};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum View {
    Home,
    Search,
    TvDetail,
    SeasonDetail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchType {
    Movie,
    Tv,
}

#[derive(Debug, Clone)]
pub struct Section {
    pub title: String,
    pub items: Vec<MediaItem>,
}

pub enum AppAction {
    HomeLoaded {
        now_playing: Result<Vec<MediaItem>>,
        popular_movies: Result<Vec<MediaItem>>,
        top_rated_movies: Result<Vec<MediaItem>>,
        popular_tv: Result<Vec<MediaItem>>,
        top_rated_tv: Result<Vec<MediaItem>>,
    },
    SearchLoaded(Result<Vec<MediaItem>>),
    TvDetailLoaded(Result<TvDetail>),
    SeasonDetailLoaded(Result<SeasonDetail>),
}

pub struct App {
    pub view: View,
    pub tmdb: Arc<TmdbClient>,
    pub action_tx: mpsc::UnboundedSender<AppAction>,

    // Home: section_idx = row, item_idx = column
    pub sections: Vec<Section>,
    pub section_idx: usize,
    pub item_idx: usize,

    // Search
    pub search_query: String,
    pub search_results: Vec<MediaItem>,
    pub search_item_idx: usize,
    pub search_type: SearchType,
    pub search_input_mode: bool,

    // TV detail
    pub tv_detail: Option<TvDetail>,
    pub tv_item_idx: usize,

    // Season detail
    pub season_detail: Option<SeasonDetail>,
    pub season_item_idx: usize,
    pub current_tv_id: Option<u32>,

    // Loading
    pub loading: bool,
    pub error: Option<String>,
    pub tick: u64,

    pub should_quit: bool,
}

impl App {
    pub fn new(tmdb: TmdbClient, action_tx: mpsc::UnboundedSender<AppAction>) -> Self {
        info!("Initializing app");
        Self {
            view: View::Home,
            tmdb: Arc::new(tmdb),
            action_tx,

            sections: Vec::new(),
            section_idx: 0,
            item_idx: 0,

            search_query: String::new(),
            search_results: Vec::new(),
            search_item_idx: 0,
            search_type: SearchType::Movie,
            search_input_mode: false,

            tv_detail: None,
            tv_item_idx: 0,

            season_detail: None,
            season_item_idx: 0,
            current_tv_id: None,

            loading: true,
            error: None,
            tick: 0,
            should_quit: false,
        }
    }

    pub fn load_home(&mut self) {
        info!("Loading home data");
        self.loading = true;
        self.error = None;
        let tmdb = self.tmdb.clone();
        let tx = self.action_tx.clone();

        tokio::spawn(async move {
            let now_playing = tmdb.now_playing().await;
            tokio::time::sleep(Duration::from_millis(500)).await;
            let popular_movies = tmdb.popular_movies().await;
            tokio::time::sleep(Duration::from_millis(500)).await;
            let top_rated_movies = tmdb.top_rated_movies().await;
            tokio::time::sleep(Duration::from_millis(500)).await;
            let popular_tv = tmdb.popular_tv().await;
            tokio::time::sleep(Duration::from_millis(500)).await;
            let top_rated_tv = tmdb.top_rated_tv().await;

            info!("All home API calls completed, sending results");
            let _ = tx.send(AppAction::HomeLoaded {
                now_playing,
                popular_movies,
                top_rated_movies,
                popular_tv,
                top_rated_tv,
            });
        });
    }

    pub fn search(&mut self) {
        if self.search_query.is_empty() {
            return;
        }
        info!("Searching: '{}' (type: {:?})", self.search_query, self.search_type);
        self.loading = true;
        self.error = None;
        let tmdb = self.tmdb.clone();
        let query = self.search_query.clone();
        let search_type = self.search_type.clone();
        let tx = self.action_tx.clone();

        tokio::spawn(async move {
            let result = match search_type {
                SearchType::Movie => tmdb.search_movies(&query).await,
                SearchType::Tv => tmdb.search_tv(&query).await,
            };
            let _ = tx.send(AppAction::SearchLoaded(result));
        });
    }

    pub fn select_item(&mut self) {
        match self.view {
            View::Home => {
                let items = self.current_section_items();
                if let Some(item) = items.get(self.item_idx) {
                    let id = item.id;
                    let title = &self.sections[self.section_idx].title;
                    info!("Selected item: '{}' (id: {}) from section '{}'", item.display_title(), id, title);
                    if title.contains("TV") {
                        self.fetch_tv_detail(id);
                    } else {
                        open_vidsrc_movie(id, item.display_title());
                    }
                }
            }
            View::Search => {
                if let Some(item) = self.search_results.get(self.search_item_idx) {
                    let id = item.id;
                    info!("Selected search result: '{}' (id: {}, type: {:?})", item.display_title(), id, item.media_type);
                    match self.search_type {
                        SearchType::Movie => {
                            open_vidsrc_movie(id, item.display_title());
                        }
                        SearchType::Tv => {
                            self.fetch_tv_detail(id);
                        }
                    }
                }
            }
            View::TvDetail => {
                if let Some(ref detail) = self.tv_detail {
                    if let Some(season) = detail.seasons.get(self.tv_item_idx) {
                        info!("Selected season {} of '{}'", season.season_number, detail.name);
                        self.current_tv_id = Some(detail.id);
                        self.fetch_season_detail(detail.id, season.season_number);
                    }
                }
            }
            View::SeasonDetail => {
                if let Some(ref detail) = self.season_detail {
                    if let Some(episode) = detail.episodes.get(self.season_item_idx) {
                        if let Some(tv_id) = self.current_tv_id {
                            info!("Selected episode: '{}' S{}E{}", detail.name, episode.season_number, episode.episode_number);
                            open_vidsrc_episode(tv_id, episode.season_number, episode.episode_number, &detail.name);
                        }
                    }
                }
            }
        }
    }

    fn fetch_tv_detail(&mut self, id: u32) {
        info!("Fetching TV detail for id {}", id);
        self.loading = true;
        self.error = None;
        let tmdb = self.tmdb.clone();
        let tx = self.action_tx.clone();

        tokio::spawn(async move {
            let result = tmdb.tv_detail(id).await;
            let _ = tx.send(AppAction::TvDetailLoaded(result));
        });
    }

    fn fetch_season_detail(&mut self, tv_id: u32, season_number: u32) {
        info!("Fetching season {} detail for TV id {}", season_number, tv_id);
        self.loading = true;
        self.error = None;
        let tmdb = self.tmdb.clone();
        let tx = self.action_tx.clone();

        tokio::spawn(async move {
            let result = tmdb.season_detail(tv_id, season_number).await;
            let _ = tx.send(AppAction::SeasonDetailLoaded(result));
        });
    }

    pub fn handle_action(&mut self, action: AppAction) {
        match action {
            AppAction::HomeLoaded {
                now_playing,
                popular_movies,
                top_rated_movies,
                popular_tv,
                top_rated_tv,
            } => {
                info!("Processing HomeLoaded action");
                let mut errors = Vec::new();

                self.sections.clear();
                match now_playing {
                    Ok(items) if !items.is_empty() => {
                        info!("Adding Now Playing section ({} items)", items.len());
                        self.sections.push(Section { title: "Now Playing".into(), items });
                    },
                    Err(e) => {
                        warn!("Now Playing failed: {}", e);
                        errors.push(format!("Now Playing: {}", e));
                    },
                    _ => debug!("Now Playing returned empty"),
                }
                match popular_movies {
                    Ok(items) if !items.is_empty() => {
                        info!("Adding Popular Movies section ({} items)", items.len());
                        self.sections.push(Section { title: "Popular Movies".into(), items });
                    },
                    Err(e) => {
                        warn!("Popular Movies failed: {}", e);
                        errors.push(format!("Popular Movies: {}", e));
                    },
                    _ => debug!("Popular Movies returned empty"),
                }
                match top_rated_movies {
                    Ok(items) if !items.is_empty() => {
                        info!("Adding Top Rated Movies section ({} items)", items.len());
                        self.sections.push(Section { title: "Top Rated Movies".into(), items });
                    },
                    Err(e) => {
                        warn!("Top Rated Movies failed: {}", e);
                        errors.push(format!("Top Rated Movies: {}", e));
                    },
                    _ => debug!("Top Rated Movies returned empty"),
                }
                match popular_tv {
                    Ok(items) if !items.is_empty() => {
                        info!("Adding Popular TV section ({} items)", items.len());
                        self.sections.push(Section { title: "Popular TV".into(), items });
                    },
                    Err(e) => {
                        warn!("Popular TV failed: {}", e);
                        errors.push(format!("Popular TV: {}", e));
                    },
                    _ => debug!("Popular TV returned empty"),
                }
                match top_rated_tv {
                    Ok(items) if !items.is_empty() => {
                        info!("Adding Top Rated TV section ({} items)", items.len());
                        self.sections.push(Section { title: "Top Rated TV".into(), items });
                    },
                    Err(e) => {
                        warn!("Top Rated TV failed: {}", e);
                        errors.push(format!("Top Rated TV: {}", e));
                    },
                    _ => debug!("Top Rated TV returned empty"),
                }

                info!("Home loaded: {} sections total", self.sections.len());
                if !errors.is_empty() {
                    self.error = Some(errors.join("; "));
                    warn!("Errors during home load: {}", self.error.as_ref().unwrap());
                }
                self.loading = false;
            }
            AppAction::SearchLoaded(result) => match result {
                Ok(items) => {
                    info!("Search returned {} results", items.len());
                    self.search_results = items;
                    self.search_item_idx = 0;
                    self.loading = false;
                }
                Err(e) => {
                    error!("Search failed: {}", e);
                    self.error = Some(e.to_string());
                    self.search_results.clear();
                    self.loading = false;
                }
            },
            AppAction::TvDetailLoaded(Ok(detail)) => {
                info!("TV detail loaded: '{}' ({} seasons)", detail.name, detail.seasons.len());
                self.current_tv_id = Some(detail.id);
                self.tv_detail = Some(detail);
                self.tv_item_idx = 0;
                self.view = View::TvDetail;
                self.loading = false;
            }
            AppAction::TvDetailLoaded(Err(e)) => {
                error!("TV detail load failed: {}", e);
                self.error = Some(e.to_string());
                self.loading = false;
            }
            AppAction::SeasonDetailLoaded(Ok(detail)) => {
                info!("Season detail loaded: '{}' ({} episodes)", detail.name, detail.episodes.len());
                self.season_detail = Some(detail);
                self.season_item_idx = 0;
                self.view = View::SeasonDetail;
                self.loading = false;
            }
            AppAction::SeasonDetailLoaded(Err(e)) => {
                error!("Season detail load failed: {}", e);
                self.error = Some(e.to_string());
                self.loading = false;
            }
        }
    }

    pub fn current_section_items(&self) -> &[MediaItem] {
        self.sections
            .get(self.section_idx)
            .map(|s| s.items.as_slice())
            .unwrap_or(&[])
    }

    pub fn go_back(&mut self) {
        debug!("Going back from {:?}", self.view);
        match self.view {
            View::Search => {
                self.view = View::Home;
                self.search_query.clear();
                self.search_results.clear();
                self.search_input_mode = false;
            }
            View::TvDetail => {
                self.view = View::Home;
                self.tv_detail = None;
                self.current_tv_id = None;
            }
            View::SeasonDetail => {
                self.view = View::TvDetail;
                self.season_detail = None;
            }
            View::Home => {}
        }
    }

    pub fn move_left(&mut self) {
        if self.item_idx > 0 {
            self.item_idx -= 1;
        }
    }

    pub fn move_right(&mut self) {
        let len = self.current_section_items().len();
        if len > 0 && self.item_idx + 1 < len {
            self.item_idx += 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.section_idx + 1 < self.sections.len() {
            self.section_idx += 1;
            self.item_idx = 0;
        }
    }

    pub fn move_up(&mut self) {
        if self.section_idx > 0 {
            self.section_idx -= 1;
            self.item_idx = 0;
        }
    }
}

fn open_vidsrc_movie(tmdb_id: u32, title: &str) {
    let url = format!("https://vidsrcme.ru/embed/movie?tmdb={}", tmdb_id);
    info!("Opening Vidsrc movie URL: {}", url);
    match open::that(&url) {
        Ok(_) => info!("Opened browser for '{}'", title),
        Err(e) => {
            error!("Failed to open browser for '{}': {}. URL: {}", title, e, url);
            eprintln!("URL: {}", url);
        }
    }
}

fn open_vidsrc_episode(tv_id: u32, season: u32, episode: u32, title: &str) {
    let url = format!(
        "https://vidsrcme.ru/embed/tv?tmdb={}&season={}&episode={}",
        tv_id, season, episode
    );
    info!("Opening Vidsrc episode URL: {}", url);
    match open::that(&url) {
        Ok(_) => info!("Opened browser for '{}' S{}E{}", title, season, episode),
        Err(e) => {
            error!("Failed to open browser for '{}' S{}E{}: {}. URL: {}", title, season, episode, e, url);
            eprintln!("URL: {}", url);
        }
    }
}
