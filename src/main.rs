mod sherlog_core;
mod test_data;

use sherlog_core::Sherlog;

use crossterm::event::{self, Event, KeyCode};
use crossterm::{execute, terminal, Result};
use tui::backend::{Backend, CrosstermBackend};
use tui::layout;
use tui::widgets;
use tui::{Frame, Terminal};

struct App {
    core: Sherlog,
    view_offset: usize,
}

impl App {
    pub fn new(core: Sherlog) -> Self {
        App {
            core,
            view_offset: 0,
        }
    }

    pub fn scroll_up(&mut self) {
        self.view_offset = self.view_offset.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        self.view_offset = self.view_offset.saturating_add(1);
        let max_offset = self.core.line_count() - 1;
        if self.view_offset > max_offset {
            self.view_offset = max_offset;
        }
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Up => app.scroll_up(),
                KeyCode::Down => app.scroll_down(),
                _ => {}
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    // Create two chunks for text view and command bar
    let chunks = layout::Layout::default()
        .direction(layout::Direction::Vertical)
        .constraints([layout::Constraint::Min(1), layout::Constraint::Length(1)].as_ref())
        .split(f.size());

    let paragraph = widgets::Paragraph::new(
        app.core
            .get_lines(app.view_offset, Some(chunks[0].height as usize))
            .join("\n"),
    );
    f.render_widget(paragraph, chunks[0]);

    let command_line = widgets::Paragraph::new(":");
    f.render_widget(command_line, chunks[1]);
}

fn main() -> Result<()> {
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

    // create app and run it
    let app = App::new(Sherlog::new(&test_data::SAMPLE_LOG));
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
