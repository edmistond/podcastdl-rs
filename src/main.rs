use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState},
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

#[derive(Debug)]
struct Episode {
    title: Option<String>,
    pub_date: Option<DateTime<Utc>>,
}

struct App {
    episodes: Vec<Episode>,
    list_state: ListState,
}

impl App {
    fn new(episodes: Vec<Episode>) -> App {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        App {
            episodes,
            list_state,
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
    if let Some(title) = feed.title {
        println!("Feed Title: {}", title.content);
    }

    // Print each entry's title
    println!("\nMost Recent 50 Episodes:");

    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // Create app state
    let mut app = App::new(episodes);

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
            // Only respond to key press events, not key release
            if key.kind == event::KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Up => app.previous(),
                    KeyCode::Down => app.next(),
                    KeyCode::Char('d') => {
                        if let Err(e) = download_selected_episode(app) {
                            // Print error below the UI
                            disable_raw_mode()?;
                            stdout().execute(LeaveAlternateScreen)?;
                            println!("Download error: {}", e);
                            return Ok(());
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

fn download_selected_episode(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let selected_idx = app.list_state.selected().ok_or("No episode selected")?;
    let episode = &app.episodes[selected_idx];
    
    // Get the enclosure URL from the feed
    let content = std::fs::read_to_string("techmeme-ridehome.rss")?;
    let feed = feed_rs::parser::parse(content.as_bytes())?;
    let entry = feed.entries.get(selected_idx).ok_or("Entry not found")?;
    
    // Find the first enclosure (attachment) URL
    let url = entry.links.iter()
        .find(|link| link.rel.as_deref() == Some("enclosure"))
        .map(|link| &link.href)
        .ok_or("No download URL found")?;

    // Create filename from title or use a default
    let filename = episode.title
        .as_ref()
        .map(|t| sanitize_filename(t))
        .unwrap_or_else(|| format!("episode_{}.mp3", selected_idx));

    // Initialize curl
    let mut easy = curl::easy::Easy::new();
    easy.url(url)?;
    easy.follow_location(true)?;
    easy.max_redirections(20)?;

    // Open file for writing
    let mut file = std::fs::File::create(&filename)?;
    
    // Configure curl to write to our file
    {
        let mut transfer = easy.transfer();
        transfer.write_function(|data| {
            file.write_all(data).unwrap();
            Ok(data.len())
        })?;
        transfer.perform()?;
    }

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

    let selected_text = format!("Selected: {:?}", app.list_state.selected());
    let list = List::new(items)
        .block(Block::default().title(selected_text).borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");

    f.render_stateful_widget(
        list,
        f.area(),
        &mut app.list_state
    );
}
