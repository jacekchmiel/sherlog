mod app;
mod sherlog_core;
mod tui_widgets;

use std::path::Path;

use app::{handle_event, render_ui, App};
use sherlog_core::Sherlog;

use clap::Parser;
use crossterm::event;
use crossterm::{execute, terminal, Result};
use tui::backend::{Backend, CrosstermBackend};
use tui::Terminal;

fn run_app<B: Backend + std::io::Write>(terminal: &mut Terminal<B>, mut app: App) -> Result<()> {
    loop {
        terminal.draw(|f| render_ui(f, &mut app))?;
        if let Some(_) = app.cursor() {
            terminal.show_cursor()?;
        } else {
            terminal.hide_cursor()?;
        }
        handle_event(&mut app, event::read()?);

        if app.wants_quit {
            return Ok(());
        }
    }
}

/// Log investigator. Helps analyzing textual log files with rich (not yet) filtering options, search (not yet) and
/// storing investigation session ready to resume after having a cup of coffee (also not yet implemented).
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file
    #[arg(value_name = "LOG_FILE")]
    input: String,
}

fn restore_terminal() -> Result<()> {
    terminal::disable_raw_mode()?;
    crossterm::execute!(
        std::io::stdout(),
        event::DisableMouseCapture,
        terminal::LeaveAlternateScreen,
    )?;
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    let log_data = std::fs::read_to_string(&args.input)?;
    let filename = Path::new(&args.input)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or(String::from("invalid_filename"));

    // setup terminal
    terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        event::EnableMouseCapture
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Register panic hook
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic| {
        restore_terminal().unwrap();
        original_hook(panic);
    }));

    // main application loop
    let app = App::new(Sherlog::new(&log_data), filename, terminal.size()?);
    let res = run_app(&mut terminal, app);

    restore_terminal()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}
