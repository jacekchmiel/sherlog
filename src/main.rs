mod sherlog_core;
use sherlog_core::Sherlog;

const SAMPLE_LOG: &'static str = r#"[2022-12-01 15:23:01] [INFO] FictitiousApp started
[2022-12-01 15:23:02] [DEBUG] Connecting to database...
[2022-12-01 15:23:03] [DEBUG] Connection established
[2022-12-01 15:23:03] [DEBUG] Loading configuration file...
[2022-12-01 15:23:04] [INFO] Configuration loaded
[2022-12-01 15:23:04] [INFO] Starting server on localhost:8080
[2022-12-01 15:23:05] [INFO] Server started
[2022-12-01 15:24:01] [INFO] User "johndoe" logged in
[2022-12-01 15:25:01] [WARNING] User "johndoe" attempted to access unauthorized resource
[2022-12-01 15:26:01] [ERROR] Internal server error: NullPointerException
[2022-12-01 15:27:01] [INFO] User "janedoe" logged in
[2022-12-01 15:28:01] [INFO] User "janedoe" accessed resource "/profile"
[2022-12-01 15:29:01] [INFO] User "janedoe" accessed resource "/settings"
[2022-12-01 15:30:01] [INFO] User "janedoe" logged out
[2022-12-01 15:31:01] [INFO] User "testuser" logged in
[2022-12-01 15:32:01] [INFO] User "testuser" accessed resource "/dashboard"
[2022-12-01 15:33:01] [WARNING] User "testuser" has high number of failed login attempts
[2022-12-01 15:34:01] [INFO] User "testuser" logged out
[2022-12-01 15:35:01] [INFO] FictitiousApp shutting down
[2022-12-01 15:35:02] [DEBUG] Disconnecting from database...
[2022-12-01 15:35:03] [DEBUG] Connection closed
[2022-12-01 15:35:03] [INFO] FictitiousApp stopped
"#;

use std::io::{stdout, Write, Stdout};
use crossterm::{
    terminal, cursor, style::{self, Stylize}, Result, execute, queue
};
use crossterm::event::{read, Event, KeyEvent, KeyCode};


struct TextView {
    row: usize,
    text: String,
    line_cnt: usize,
    height: usize,
}

impl TextView {
    pub fn new(text: String, terminal_height: usize) -> Self {
        let line_cnt = text.lines().count();
        TextView {
            row: 0,
            text,
            line_cnt,
            height: terminal_height - 1,
        }
    }

    pub fn draw(&self, stdout: &mut Stdout) -> Result<()> {
        let lines_to_draw = (self.height) as usize;
        queue!(stdout, cursor::MoveToRow(0), cursor::MoveToColumn(0))?;
        //TODO: O(1) way of drawing slice of text
        for line in self.text.lines().skip(self.row).take(lines_to_draw) {
            queue!(stdout, style::Print(line))?;
            queue!(stdout, cursor::MoveDown(1), cursor::MoveToColumn(0))?;
        }
    
        queue!(stdout, cursor::MoveToRow(self.height as u16), cursor::MoveToColumn(0))?;
        if self.is_at_end() {
            queue!(stdout, style::Print("(END)"))?;
        } else {
            queue!(stdout, style::Print(":"))?;
        }
        queue!(stdout, cursor::Show)?;
        Ok(())
    }

    pub fn move_up(&mut self) {
        self.row = self.row.saturating_sub(1);
    }

    pub fn move_down(&mut self) {
        if self.row < self.line_cnt - self.height {
            self.row += 1;
        }
    }

    pub fn is_at_end(&self) -> bool {
        self.row == self.line_cnt - self.height
    }

}

fn main() -> Result<()> {
    let mut stdout = stdout();
    terminal::enable_raw_mode()?;
    let (_, height) = terminal::size()?;
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;
    
    let mut text = TextView::new(SAMPLE_LOG.to_string(), height as usize);

    loop {
        queue!(stdout, terminal::Clear(terminal::ClearType::All))?;
        text.draw(&mut stdout)?;
        stdout.flush()?;
        match read()? {
            Event::Key(KeyEvent{code: KeyCode::Char('q'), ..}) => {break;}
            Event::Key(KeyEvent{code: KeyCode::Up, ..}) => {text.move_up()}
            Event::Key(KeyEvent{code: KeyCode::Down, ..}) => {text.move_down()}
            _ => {}
        }
    }

    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}