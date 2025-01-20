use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use std::io::{self, stdout, Write};
use chrono::{DateTime, Utc};
use curl;
use feed_rs::model::Feed;

#[derive(Debug)]
struct Episode {
    title: Option<String>,
    pub_date: Option<DateTime<Utc>>,
}

struct App {
    episodes: Vec<Episode>,
    list_state: ListState,
    status_message: Option<String>,
    feed: Feed,
}

impl App {
    fn new(episodes: Vec<Episode>, feed: Feed) -> App {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        App {
            episodes,
            list_state,
            status_message: None,
            feed,
        }
    }

    fn next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => (i + 1).min(self.episodes.len() - 1),
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => i.saturating_sub(1),
            None => 0,
        };
        self.list_state.select(Some(i));
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string("techmeme-ridehome.rss")?;
    let feed = feed_rs::parser::parse(content.as_bytes())?;
    
    let episodes: Vec<Episode> = feed.entries.iter().map(|entry| {
        Episode {
            title: entry.title.as_ref().map(|t| t.content.clone()),
            pub_date: entry.published,
        }
    }).collect();

    // Print feed title if available
    if let Some(ref title) = feed.title {
        println!("Feed Title: {}", title.content);
    }

    // Print each entry's title
    println!("\nMost Recent 50 Episodes:");

    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // Create app state with feed
    let mut app = App::new(episodes, feed);

    // Run the application loop
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Up => app.previous(),
                    KeyCode::Down => app.next(),
                    KeyCode::Char('d') => {
                        match download_selected_episode(app, terminal) {
                            Ok(_) => {} // Status message is already set in the function
                            Err(e) => {
                                app.status_message = Some(format!("Error: {}", e));
                                terminal.draw(|f| ui(f, app))?;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

fn download_selected_episode(app: &mut App, terminal: &mut Terminal<impl Backend>) -> Result<(), Box<dyn std::error::Error>> {
    let selected_idx = app.list_state.selected().ok_or("No episode selected")?;
    let episode = &app.episodes[selected_idx];
    let entry = app.feed.entries.get(selected_idx).ok_or("Entry not found")?;
    
    let url = entry.media.iter()
        .flat_map(|m| m.content.iter())
        .filter_map(|c| c.url.as_ref())
        .next()
        .ok_or("No download URL found")?;

    let filename = episode.title
        .as_ref()
        .map(|t| sanitize_filename(t))
        .unwrap_or_else(|| format!("episode_{}.mp3", selected_idx));

    let mut easy = curl::easy::Easy::new();
    easy.url(url.as_str())?;
    easy.follow_location(true)?;
    easy.max_redirections(20)?;
    easy.progress(true)?;

    let mut file = std::fs::File::create(&filename)?;
    
    app.status_message = Some(format!("Downloading {}... 0%", filename));
    terminal.draw(|f| ui(f, app))?;
    
    {
        let mut transfer = easy.transfer();
        
        transfer.progress_function(|total, current, _, _| {
            if total > 0.0 {
                let percentage = (current / total * 100.0) as i32;
                app.status_message = Some(format!("Downloading {}... {}%", filename, percentage));
                // Force a redraw of the terminal
                terminal.draw(|f| ui(f, app)).unwrap();
            }
            true
        })?;

        transfer.write_function(|data| {
            file.write_all(data).unwrap();
            Ok(data.len())
        })?;

        transfer.perform()?;
    }

    app.status_message = Some(format!("Downloaded {}", filename));
    terminal.draw(|f| ui(f, app))?;
    Ok(())
}

fn sanitize_filename(filename: &str) -> String {
    // Replace invalid filename characters with underscores
    let invalid_chars = r#"/\?%*:|"<>"#;
    let mut sanitized = filename.to_string();
    for c in invalid_chars.chars() {
        sanitized = sanitized.replace(c, "_");
    }
    // Add .mp3 extension if not present
    if !sanitized.ends_with(".mp3") {
        sanitized.push_str(".mp3");
    }
    sanitized
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Status line
            Constraint::Min(0),     // List
        ])
        .split(f.area());

    // Render status message if any
    if let Some(msg) = &app.status_message {
        let status = Line::from(msg.as_str());
        f.render_widget(Paragraph::new(status), chunks[0]);
    }

    let items: Vec<ListItem> = app
        .episodes
        .iter()
        .enumerate()
        .map(|(i, episode)| {
            let title = episode.title.as_deref().unwrap_or("Untitled Episode");
            let date = episode.pub_date
                .map(|d| d.format("%d %b %Y").to_string())
                .unwrap_or_else(|| "Unknown date".to_string());
            
            ListItem::new(format!("{}: {} ({})", i, title, date))
        })
        .collect();

    // Get the media filename for the selected episode
    let title = match app.list_state.selected() {
        Some(idx) => {
            app.feed.entries.get(idx)
                .and_then(|e| {
                    if e.media.iter().any(|m| !m.content.is_empty()) {
                        let episode = &app.episodes[idx];
                        episode.title.as_ref()
                            .map(|t| sanitize_filename(t))
                            .or_else(|| Some(format!("episode_{}.mp3", idx)))
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "No media found".to_string())
        }
        None => "No episode selected".to_string(),
    };

    let list = List::new(items)
        .block(Block::default().title(title).borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");

    f.render_stateful_widget(list, chunks[1], &mut app.list_state);
}
