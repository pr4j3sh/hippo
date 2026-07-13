use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use serde::Deserialize;
use std::time::Duration;

const BASE_URL: &str = "https://api.themoviedb.org/3";

#[derive(Debug, Clone)]
pub struct TmdbClient {
    token: String,
    http: reqwest::Client,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct MediaItem {
    pub id: u32,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub overview: String,
    #[serde(default)]
    pub poster_path: Option<String>,
    #[serde(default)]
    pub vote_average: f64,
    #[serde(default)]
    pub vote_count: u32,
    #[serde(default)]
    pub release_date: Option<String>,
    #[serde(default)]
    pub first_air_date: Option<String>,
    #[serde(default)]
    pub media_type: MediaType,
    #[serde(default)]
    pub genre_ids: Vec<u32>,
    #[serde(default)]
    pub original_language: String,
}

impl MediaItem {
    pub fn display_title(&self) -> &str {
        if self.title.is_empty() {
            &self.name
        } else {
            &self.title
        }
    }

    pub fn display_date(&self) -> &str {
        self.release_date
            .as_deref()
            .or(self.first_air_date.as_deref())
            .unwrap_or("N/A")
    }

    pub fn display_genre(&self) -> &str {
        self.genre_ids.first().map_or("", |id| match id {
            28 => "Action",
            12 => "Adventure",
            16 => "Animation",
            35 => "Comedy",
            80 => "Crime",
            99 => "Documentary",
            18 => "Drama",
            10751 => "Family",
            14 => "Fantasy",
            36 => "History",
            27 => "Horror",
            10402 => "Music",
            9648 => "Mystery",
            10749 => "Romance",
            878 => "Sci-Fi",
            10770 => "TV Movie",
            53 => "Thriller",
            10752 => "War",
            37 => "Western",
            _ => "",
        })
    }

    pub fn display_language(&self) -> &str {
        match self.original_language.as_str() {
            "en" => "EN",
            "es" => "ES",
            "fr" => "FR",
            "de" => "DE",
            "ja" => "JA",
            "ko" => "KO",
            "zh" => "ZH",
            "pt" => "PT",
            "it" => "IT",
            "ru" => "RU",
            _ => "",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct TmdbResponse {
    pub page: u32,
    pub results: Vec<MediaItem>,
    pub total_pages: u32,
    pub total_results: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SeasonInfo {
    pub season_number: u32,
    pub name: String,
    pub episode_count: u32,
    pub overview: String,
    #[serde(default)]
    pub air_date: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TvDetail {
    pub id: u32,
    pub name: String,
    pub overview: String,
    pub seasons: Vec<SeasonInfo>,
    pub number_of_seasons: u32,
    pub number_of_episodes: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EpisodeInfo {
    pub episode_number: u32,
    pub season_number: u32,
    pub name: String,
    pub overview: String,
    #[serde(default)]
    pub air_date: Option<String>,
    #[serde(default)]
    pub vote_average: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct SeasonDetail {
    pub season_number: u32,
    pub name: String,
    pub episodes: Vec<EpisodeInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum MediaType {
    #[serde(rename = "movie")]
    Movie,
    #[serde(rename = "tv")]
    Tv,
    #[serde(other)]
    Unknown,
}

impl Default for MediaType {
    fn default() -> Self {
        MediaType::Unknown
    }
}

impl TmdbClient {
    pub fn from_env() -> Result<Self> {
        let token = std::env::var("TMDB_API_KEY")
            .context("TMDB_API_KEY environment variable not set")?;
        info!("TmdbClient initialized with token ending in ...{}", &token[token.len().saturating_sub(4)..]);
        Ok(Self {
            token,
            http: reqwest::Client::new(),
        })
    }

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.token)
    }

    pub async fn now_playing(&self) -> Result<Vec<MediaItem>> {
        info!("Fetching now playing movies");
        let result = self.get_media("/movie/now_playing").await;
        match &result {
            Ok(items) => info!("Now playing: {} movies loaded", items.len()),
            Err(e) => error!("Failed to fetch now playing: {}", e),
        }
        result
    }

    pub async fn popular_movies(&self) -> Result<Vec<MediaItem>> {
        info!("Fetching popular movies");
        let result = self.get_media("/movie/popular").await;
        match &result {
            Ok(items) => info!("Popular movies: {} loaded", items.len()),
            Err(e) => error!("Failed to fetch popular movies: {}", e),
        }
        result
    }

    pub async fn top_rated_movies(&self) -> Result<Vec<MediaItem>> {
        info!("Fetching top rated movies");
        let result = self.get_media("/movie/top_rated").await;
        match &result {
            Ok(items) => info!("Top rated movies: {} loaded", items.len()),
            Err(e) => error!("Failed to fetch top rated movies: {}", e),
        }
        result
    }

    pub async fn popular_tv(&self) -> Result<Vec<MediaItem>> {
        info!("Fetching popular TV shows");
        let result = self.get_media("/tv/popular").await;
        match &result {
            Ok(items) => info!("Popular TV: {} loaded", items.len()),
            Err(e) => error!("Failed to fetch popular TV: {}", e),
        }
        result
    }

    pub async fn top_rated_tv(&self) -> Result<Vec<MediaItem>> {
        info!("Fetching top rated TV shows");
        let result = self.get_media("/tv/top_rated").await;
        match &result {
            Ok(items) => info!("Top rated TV: {} loaded", items.len()),
            Err(e) => error!("Failed to fetch top rated TV: {}", e),
        }
        result
    }

    pub async fn search_movies(&self, query: &str) -> Result<Vec<MediaItem>> {
        info!("Searching movies: '{}'", query);
        let url = format!(
            "{}/search/movie?query={}&include_adult=false&language=en-US&page=1",
            BASE_URL,
            urlencoding::encode(query)
        );
        let result: Result<TmdbResponse> = self.get(&url).await;
        let result = result.map(|r| r.results);
        match &result {
            Ok(items) => info!("Movie search '{}': {} results", query, items.len()),
            Err(e) => error!("Movie search '{}' failed: {}", query, e),
        }
        result
    }

    pub async fn search_tv(&self, query: &str) -> Result<Vec<MediaItem>> {
        info!("Searching TV: '{}'", query);
        let url = format!(
            "{}/search/tv?query={}&include_adult=false&language=en-US&page=1",
            BASE_URL,
            urlencoding::encode(query)
        );
        let result: Result<TmdbResponse> = self.get(&url).await;
        let result = result.map(|r| r.results);
        match &result {
            Ok(items) => info!("TV search '{}': {} results", query, items.len()),
            Err(e) => error!("TV search '{}' failed: {}", query, e),
        }
        result
    }

    pub async fn tv_detail(&self, id: u32) -> Result<TvDetail> {
        info!("Fetching TV detail for id {}", id);
        let url = format!("{}/tv/{}?language=en-US", BASE_URL, id);
        let result: Result<TvDetail> = self.get(&url).await;
        match &result {
            Ok(detail) => info!("TV detail '{}': {} seasons", detail.name, detail.seasons.len()),
            Err(e) => error!("Failed to fetch TV detail for id {}: {}", id, e),
        }
        result
    }

    pub async fn season_detail(&self, tv_id: u32, season_number: u32) -> Result<SeasonDetail> {
        info!("Fetching season {} detail for TV id {}", season_number, tv_id);
        let url = format!(
            "{}/tv/{}/season/{}?language=en-US",
            BASE_URL, tv_id, season_number
        );
        let result: Result<SeasonDetail> = self.get(&url).await;
        match &result {
            Ok(detail) => info!("Season {} '{}': {} episodes", season_number, detail.name, detail.episodes.len()),
            Err(e) => error!("Failed to fetch season {} for TV id {}: {}", season_number, tv_id, e),
        }
        result
    }

    async fn get_media(&self, path: &str) -> Result<Vec<MediaItem>> {
        let url = format!("{}{}?language=en-US&page=1", BASE_URL, path);
        let resp: TmdbResponse = self.get(&url).await
            .context(format!("Failed to get media from {}", path))?;
        Ok(resp.results)
    }

    async fn get<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<T> {
        let mut last_err = None;
        for attempt in 1..=3 {
            debug!("GET {} (attempt {})", url, attempt);
            match self.try_get(url).await {
                Ok(val) => return Ok(val),
                Err(e) => {
                    warn!("Attempt {} failed for {}: {}", attempt, url, e);
                    last_err = Some(e);
                    if attempt < 3 {
                        tokio::time::sleep(Duration::from_millis(1000 * attempt as u64)).await;
                    }
                }
            }
        }
        Err(last_err.unwrap())
    }

    async fn try_get<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<T> {
        let resp = self
            .http
            .get(url)
            .header("Authorization", self.auth_header())
            .header("accept", "application/json")
            .send()
            .await
            .context(format!("HTTP request failed for {}", url))?;

        let status = resp.status();
        debug!("Response status: {} for {}", status, url);

        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            warn!("HTTP {} from {}: {}", status, url, body);
            anyhow::bail!("HTTP {} from {}: {}", status, url, body);
        }

        resp.json::<T>()
            .await
            .context(format!("Failed to parse JSON from {}", url))
    }
}
