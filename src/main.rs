mod app;
mod sherlog_core;
mod tui_widgets;

use std::path::Path;

use app::App;
use sherlog_core::Sherlog;

use clap::Parser;
use crossterm::event::{self, Event, KeyModifiers};
use crossterm::{execute, terminal, Result};
use tui::backend::{Backend, CrosstermBackend};
use tui::{layout, Frame, Terminal};

fn handle_event(app: &mut App, event: Event) {
    match event {
        Event::Key(key) => {
            app.on_key(key);
        }
        Event::Mouse(mouse) => match mouse.kind {
            event::MouseEventKind::ScrollDown if mouse.modifiers == KeyModifiers::CONTROL => {
                app.view.scroll_down(10);
            }
            event::MouseEventKind::ScrollUp if mouse.modifiers == KeyModifiers::CONTROL => {
                app.view.scroll_up(10);
            }
            event::MouseEventKind::ScrollDown => app.view.scroll_down(1),
            event::MouseEventKind::ScrollUp => app.view.scroll_up(1),
            _ => {}
        },
        _ => (),
    }
}

fn render_ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    // Create two chunks for text view and command bar
    let chunks = layout::Layout::default()
        .direction(layout::Direction::Vertical)
        .constraints([layout::Constraint::Min(1), layout::Constraint::Length(1)].as_ref())
        .split(f.size());

    // TODO: line retrieval shouldn't be done in render...
    let lines = app
        .core
        .get_lines(app.view.y, Some(chunks[0].height as usize));
    let last_line_shown = lines.last().map(|l| l.line_num);

    f.render_widget(app.view.widget(lines), chunks[0]);

    f.render_widget(app.status_line.widget(last_line_shown), chunks[1]);

    if let Some((widget, mut state)) = app.filters.widget() {
        let overlay_area = layout::Layout::default()
            .horizontal_margin(5)
            .vertical_margin(1)
            .direction(layout::Direction::Vertical)
            .constraints([layout::Constraint::Min(0), layout::Constraint::Ratio(1, 2)])
            .split(f.size())[1];

        f.render_stateful_widget(widget, overlay_area, &mut state);
    }

    if let Some(x) = app.status_line.cursor_x() {
        f.set_cursor(x, chunks[1].y);
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<()> {
    loop {
        terminal.draw(|f| render_ui(f, &mut app))?;
        if let Some(_) = app.status_line.cursor_x() {
            // terminal.set_cursor(x, y)?;
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

    // main application loop
    let app = App::new(Sherlog::new(&log_data), filename);
    let res = run_app(&mut terminal, app);

    // restore terminal
    terminal::disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        terminal::LeaveAlternateScreen,
        event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}
