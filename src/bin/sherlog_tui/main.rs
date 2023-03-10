mod app;
mod filter_list;
mod status_line;
mod text_area;
mod ty;
mod widgets;

use std::path::Path;

use anyhow::Result;
use app::App;
use clap::Parser;
use crossterm::event;
use crossterm::{execute, terminal};
use log::{info, LevelFilter};
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Root};
use sherlog::Sherlog;
use tui::backend::{Backend, CrosstermBackend};
use tui::Terminal;

fn run_app<B: Backend + std::io::Write>(terminal: &mut Terminal<B>, mut app: App) -> Result<()> {
    loop {
        terminal.draw(|f| app.render(f))?;
        app.handle_event(event::read()?);
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

    /// Provide debug log during execution
    #[arg(short, long)]
    debug: bool,
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

    // setup logging
    if args.debug {
        let logfile = FileAppender::builder()
            // .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
            .build("sherlog.log")?;

        let config = log4rs::Config::builder()
            .appender(Appender::builder().build("logfile", Box::new(logfile)))
            .build(
                Root::builder()
                    .appender("logfile")
                    .build(LevelFilter::Debug),
            )?;

        log4rs::init_config(config)?;
        info!("Sherlog started with {}", args.input);
    }

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
        println!("{err:?}")
    }

    Ok(())
}
