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
use std::io::{self, stdout};
use chrono::{DateTime, Utc};

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
                    _ => {}
                }
            }
        }
    }
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
