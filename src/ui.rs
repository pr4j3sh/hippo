use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::{App, SearchType, Section, View};

const SECTION_HEIGHT: usize = 10;

const HIPPO_NORMAL: &str = r#"   >  <
   ,.--'  ''-. 
   (  )  ',_.' 
    Xx'xX"#;

const HIPPO_BRIGHT: &str = r#"   >  <
   ,.--'  ''-. 
   (  )  ',_.' 
    mn'mn`"#;

fn render_hippo(frame: &mut Frame, area: Rect, tick: u64) {
    let beat = (tick / 4) % 2 == 0;
    let (ascii, style) = if beat {
        (
            HIPPO_NORMAL,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        (
            HIPPO_BRIGHT,
            Style::default().fg(Color::DarkGray),
        )
    };

    let lines: Vec<Line> = ascii
        .lines()
        .map(|l| Line::from(Span::styled(l.to_string(), style)))
        .collect();

    let hippo = Paragraph::new(lines).alignment(Alignment::Center);
    let hippo_area = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(4) / 2,
        width: area.width,
        height: 4,
    };
    frame.render_widget(hippo, hippo_area);

    let dots = ".".repeat(((tick / 6) % 4) as usize);
    let loading = Paragraph::new(format!("Loading{}", dots))
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));
    let loading_area = Rect {
        x: area.x,
        y: hippo_area.y + 5,
        width: area.width,
        height: 1,
    };
    frame.render_widget(loading, loading_area);
}

pub fn ui(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Content
            Constraint::Length(1), // Help bar
        ])
        .split(area);

    match app.view {
        View::Home => render_home(frame, app, main_layout[0]),
        View::Search => render_search(frame, app, main_layout[0]),
        View::TvDetail => render_tv_detail(frame, app, main_layout[0]),
        View::SeasonDetail => render_season_detail(frame, app, main_layout[0]),
    }

    render_help_bar(frame, app, main_layout[1]);
}

fn render_home(frame: &mut Frame, app: &App, area: Rect) {
    if app.loading && app.sections.is_empty() {
        render_hippo(frame, area, app.tick);
        return;
    }

    if app.sections.is_empty() {
        let msg = if let Some(ref err) = app.error {
            format!("No data loaded: {}", err)
        } else {
            "No data available".to_string()
        };
        let empty = Paragraph::new(msg)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty, area);
        return;
    }

    let constraints: Vec<Constraint> = app
        .sections
        .iter()
        .map(|_| Constraint::Length(SECTION_HEIGHT as u16 + 2))
        .collect();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    for (i, section) in app.sections.iter().enumerate() {
        let is_active = i == app.section_idx;
        render_section(frame, section, chunks[i], is_active, app.item_idx);
    }
}

fn render_section(
    frame: &mut Frame,
    section: &Section,
    area: Rect,
    is_active: bool,
    selected_item: usize,
) {
    let border_color = if is_active {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let block = Block::default()
        .title(format!(" {} ", section.title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if section.items.is_empty() {
        return;
    }

    let card_width = 32u16;
    let gap = 2u16;
    let step = card_width + gap;

    let items_per_page = if inner.width > gap {
        ((inner.width - gap) / step) as usize
    } else {
        1
    };
    let items_per_page = items_per_page.max(1);

    for (j, item) in section.items.iter().take(items_per_page).enumerate() {
        let x = inner.x + gap + (j as u16 * step);
        let card_area = Rect {
            x,
            y: inner.y,
            width: card_width.min(inner.width.saturating_sub(x - inner.x)),
            height: inner.height,
        };

        if card_area.width < 8 {
            break;
        }

        let is_selected = is_active && j == selected_item;

        let inner_w = card_area.width.saturating_sub(4) as usize;

        let title = item.display_title();
        let rating = format!("★ {:.1}", item.vote_average);
        let votes = format!("({})", item.vote_count);
        let date = item.display_date();
        let genre = item.display_genre();
        let lang = item.display_language();
        let overview = truncate_str(&item.overview, inner_w);

        let title_style = if is_selected {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let meta_style = if is_selected {
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let mut lines: Vec<Line> = Vec::new();
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            truncate_str(&title, inner_w),
            title_style,
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(&rating, Style::default().fg(Color::Yellow)),
            Span::styled(
                format!(" {}", votes),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
        if !date.is_empty() && date != "N/A" {
            lines.push(Line::from(Span::styled(date, meta_style)));
        }
        if !genre.is_empty() {
            lines.push(Line::from(Span::styled(genre, meta_style)));
        }
        if !lang.is_empty() {
            lines.push(Line::from(Span::styled(lang, meta_style)));
        }
        if !overview.is_empty() {
            lines.push(Line::from(""));
            for line in overview.lines().take(3) {
                lines.push(Line::from(Span::styled(
                    truncate_str(line, inner_w),
                    meta_style,
                )));
            }
        }

        let paragraph = Paragraph::new(lines);
        let text_area = Rect {
            x: card_area.x + 2,
            y: card_area.y,
            width: card_area.width.saturating_sub(4),
            height: card_area.height,
        };
        frame.render_widget(paragraph, text_area);
    }
}

pub fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len > 3 {
        format!("{}...", &s[..max_len - 3])
    } else {
        s[..max_len].to_string()
    }
}

fn render_search(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search input
            Constraint::Length(2), // Search type tabs
            Constraint::Min(0),   // Results
        ])
        .split(area);

    // Search input
    let input_style = if app.search_input_mode {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let input_block = Block::default()
        .title(" Search (/ to focus, Enter to search) ")
        .borders(Borders::ALL)
        .border_style(input_style);

    let input_text = format!("{}/", app.search_query);
    let input = Paragraph::new(input_text).block(input_block);
    frame.render_widget(input, chunks[0]);

    // Search type tabs
    let movie_tab = if app.search_type == SearchType::Movie {
        Span::styled(
            " Movies ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::White),
        )
    } else {
        Span::styled(" Movies ", Style::default().fg(Color::DarkGray))
    };

    let tv_tab = if app.search_type == SearchType::Tv {
        Span::styled(
            " TV ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::White),
        )
    } else {
        Span::styled(" TV ", Style::default().fg(Color::DarkGray))
    };

    let tabs = Paragraph::new(Line::from(vec![
        movie_tab,
        Span::raw("  "),
        tv_tab,
    ]));
    frame.render_widget(tabs, chunks[1]);

    // Results
    if app.loading {
        render_hippo(frame, chunks[2], app.tick);
        return;
    }

    if app.search_results.is_empty() {
        let empty = if app.search_query.is_empty() {
            "Type a query and press Enter"
        } else {
            "No results found"
        };
        let msg = Paragraph::new(empty)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(msg, chunks[2]);
        return;
    }

    let items: Vec<ListItem> = app
        .search_results
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let is_selected = i == app.search_item_idx;
            let title_style = if is_selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let title = item.display_title();
            let date = item.display_date();
            let rating = format!("★ {:.1}", item.vote_average);
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{:<50}", truncate_str(&title, 50)),
                    title_style,
                ),
                Span::styled(
                    format!("{:>6}", rating),
                    if is_selected {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Yellow)
                    },
                ),
                Span::raw("  "),
                Span::styled(
                    date,
                    if is_selected {
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    },
                ),
            ]))
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, chunks[2]);
}

fn render_tv_detail(frame: &mut Frame, app: &App, area: Rect) {
    let detail = match &app.tv_detail {
        Some(d) => d,
        None => return,
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Show info header
            Constraint::Min(0),   // Seasons list
        ])
        .split(area);

    // Show info header
    let info_text = vec![
        Line::from(Span::styled(
            &detail.name,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::raw(""),
        Line::from(Span::styled(
            truncate_str(&detail.overview, chunks[0].width as usize - 4),
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            format!(
                "{} seasons  |  {} episodes",
                detail.number_of_seasons, detail.number_of_episodes
            ),
            Style::default().fg(Color::Yellow),
        )),
    ];
    let info = Paragraph::new(info_text)
        .block(Block::default().borders(Borders::ALL).title(" Info "));
    frame.render_widget(info, chunks[0]);

    // Seasons list
    let seasons: Vec<ListItem> = detail
        .seasons
        .iter()
        .enumerate()
        .map(|(i, season)| {
            let is_selected = i == app.tv_item_idx;
            let title_style = if is_selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let meta_style = if is_selected {
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let season_label = format!("S{:<2}", season.season_number);
            let eps_label = format!("{} eps", season.episode_count);
            let air_date = season
                .air_date
                .as_deref()
                .unwrap_or("");
            let overview = truncate_str(&season.overview, 40);
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{:<6}", season_label),
                    title_style,
                ),
                Span::styled(
                    format!("{:<28}", truncate_str(&season.name, 28)),
                    title_style,
                ),
                Span::styled(
                    format!("{:>7}", eps_label),
                    if is_selected {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Yellow)
                    },
                ),
                Span::raw("  "),
                Span::styled(air_date, meta_style),
                Span::raw("  "),
                Span::styled(overview, meta_style),
            ]))
        })
        .collect();

    let season_block = Block::default()
        .title(format!(" {} - Seasons ", detail.name))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let season_list = List::new(seasons).block(season_block);
    frame.render_widget(season_list, chunks[1]);
}

fn render_season_detail(frame: &mut Frame, app: &App, area: Rect) {
    let detail = match &app.season_detail {
        Some(d) => d,
        None => return,
    };

    let items: Vec<ListItem> = detail
        .episodes
        .iter()
        .enumerate()
        .map(|(i, ep)| {
            let is_selected = i == app.season_item_idx;
            let title_style = if is_selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let meta_style = if is_selected {
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let ep_label = format!("E{:<2}", ep.episode_number);
            let rating = format!("★ {:.1}", ep.vote_average);
            let air_date = ep.air_date.as_deref().unwrap_or("");
            let overview = truncate_str(&ep.overview, 30);
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{:<6}", ep_label),
                    title_style,
                ),
                Span::styled(
                    format!("{:<30}", truncate_str(&ep.name, 30)),
                    title_style,
                ),
                Span::styled(
                    format!("{:>6}", rating),
                    if is_selected {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Yellow)
                    },
                ),
                Span::raw("  "),
                Span::styled(air_date, meta_style),
                Span::raw("  "),
                Span::styled(overview, meta_style),
            ]))
        })
        .collect();

    let block = Block::default()
        .title(format!(" {} - Episodes ", detail.name))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn render_help_bar(frame: &mut Frame, app: &App, area: Rect) {
    let help_text = match app.view {
        View::Home => " h/l:cards  j/k:sections  Space/Enter:select  /:search  q:quit ",
        View::Search => {
            if app.search_input_mode {
                " type query  Enter:search  Esc:back  Tab:switch type "
            } else {
                " j/k:scroll  Space/Enter:select  /:new search  Tab:switch type  Esc:back  q:quit "
            }
        }
        View::TvDetail => " j/k:scroll  Space/Enter:select  h/Esc:back  q:quit ",
        View::SeasonDetail => " j/k:scroll  Space/Enter:select  h/Esc:back  q:quit ",
    };

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::White).bg(Color::Rgb(40, 40, 60)))
        .alignment(Alignment::Center);
    frame.render_widget(help, area);
}
