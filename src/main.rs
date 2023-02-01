mod app;
mod sherlog_core;
mod tui_widgets;

use std::path::Path;

use app::App;
use sherlog_core::{Sherlog, TextLine};

use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::{execute, terminal, Result};
use tui::backend::{Backend, CrosstermBackend};
use tui::style::{Color, Style};
use tui::text::Spans;
use tui::{layout, widgets, Frame, Terminal};

fn handle_event(app: &mut App, event: Event) {
    match event {
        Event::Key(key) => {
            if key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::CONTROL {
                app.wants_quit = true;
                return;
            }

            match key.code {
                KeyCode::Up => app.scroll_up(1),
                KeyCode::Down => app.scroll_down(1),
                KeyCode::Left => app.scroll_left(),
                KeyCode::Right => app.scroll_right(),
                KeyCode::Char(c) => app.on_user_input(c),
                KeyCode::Backspace => app.on_backspace(),
                KeyCode::Enter => app.on_enter(),
                KeyCode::Esc => app.on_esc(),
                KeyCode::Home => app.on_home(),
                KeyCode::End => app.on_end(),
                _ => {}
            }
        }
        Event::Mouse(mouse) => match mouse.kind {
            event::MouseEventKind::ScrollDown if mouse.modifiers == KeyModifiers::CONTROL => {
                app.scroll_down(10);
            }
            event::MouseEventKind::ScrollUp if mouse.modifiers == KeyModifiers::CONTROL => {
                app.scroll_up(10);
            }
            event::MouseEventKind::ScrollDown => app.scroll_down(1),
            event::MouseEventKind::ScrollUp => app.scroll_up(1),
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

    let lines = app
        .core
        .get_lines(app.view_offset_y, Some(chunks[0].height as usize));
    let last_line_shown = lines.last().map(|l| l.line_num).unwrap_or_default();
    let spans: Vec<Spans> = lines
        .into_iter()
        .map(|line| make_spans(line, app.view_offset_x))
        .collect();
    let mut paragraph = widgets::Paragraph::new(spans);
    if app.wrap_lines {
        paragraph = paragraph.wrap(widgets::Wrap { trim: false })
    }
    f.render_widget(paragraph, chunks[0]);

    let bottom_line = tui_widgets::StatusLine::new()
        .left(app.status.to_string())
        .right(format!("{}/{}", last_line_shown, app.core.line_count()))
        .right(app.filename.as_ref());
    f.render_widget(bottom_line, chunks[1]);
}

fn make_spans<'a>(line: TextLine<'a>, offset: usize) -> tui::text::Spans {
    let mut chars_to_remove = offset;
    let spans = line.spans.into_iter();
    spans
        .filter_map(|s| {
            if chars_to_remove >= s.content.len() {
                chars_to_remove -= s.content.len();
                None
            } else {
                let remaining = s.remove_left(chars_to_remove);
                chars_to_remove = 0;
                Some(remaining)
            }
        })
        .map(|s| make_span(s))
        .collect::<Vec<_>>()
        .into()
}

fn make_span<'a>(span: sherlog_core::Span<'a>) -> tui::text::Span<'a> {
    match span.kind {
        sherlog_core::SpanKind::Raw => tui::text::Span::raw(span.content),
        sherlog_core::SpanKind::Highlight => {
            tui::text::Span::styled(span.content, Style::default().fg(Color::Red))
        }
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<()> {
    loop {
        terminal.draw(|f| render_ui(f, &mut app))?;
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
