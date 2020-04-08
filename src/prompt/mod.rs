mod buffer;
mod char_string;
mod navigation;

use char_string::CharString;

use crate::key_bindings::{action_for, Action, KeyBindings};

pub struct Prompt {
    prompt: Option<CharString>,
    bindings: Option<KeyBindings>,
    completer: Option<String>,
    suggester: Option<String>,
}

impl Prompt {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn prompt(&mut self, prompt: Option<&str>) -> &mut Self {
        self.prompt = prompt.map(std::convert::Into::into);
        self
    }

    pub fn bindings(&mut self, bindings: Option<KeyBindings>) -> &mut Self {
        self.bindings = bindings;
        self
    }

    pub fn read_line(&self) -> Option<String> {
        let stdout = std::io::stdout();
        let mut writer = Writer::new(&stdout);
        let mut buffer = buffer::Buffer::new();

        loop {
            writer.print(&self.prompt, &buffer);
            match self.next_event() {
                Action::Write(c) => buffer.write(c),
                Action::Delete(scope) => buffer.delete(scope),
                Action::Move(movement) => buffer.move_cursor(movement),
                Action::Complete(_) => {}
                Action::Suggest(_) => {}
                Action::Noop => continue,
                Action::Accept => return Some(buffer.data()),
                Action::Cancel => return None,
            }
        }
    }

    fn next_event(&self) -> Action {
        match crossterm::event::read() {
            Ok(crossterm::event::Event::Key(e)) => action_for(self.bindings.as_ref(), e),
            Ok(_) => Action::Noop,
            Err(_) => Action::Noop,
        }
    }
}

impl Default for Prompt {
    fn default() -> Self {
        Self {
            prompt: None,
            bindings: None,
            completer: None,
            suggester: None,
        }
    }
}

struct Writer<'a> {
    lock: std::io::StdoutLock<'a>,
    row: u16,
}

impl<'a> Writer<'a> {
    fn new(stdout: &'a std::io::Stdout) -> Self {
        crossterm::terminal::enable_raw_mode();
        let row = crossterm::cursor::position().map_or(0, |pos| pos.1);
        Self { lock: stdout.lock(), row }
    }

    fn print(&mut self, prompt: &Option<CharString>, buffer: &buffer::Buffer) {
        use std::io::Write;

        crossterm::queue!(
            self.lock,
            crossterm::cursor::MoveToColumn(0),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine),
        );

        let start = if let Some(prompt) = prompt {
            crossterm::queue!(self.lock, crossterm::style::Print(prompt));
            prompt.data.len()
        } else { 0 };

        crossterm::execute!(self.lock, crossterm::style::Print(buffer.data()), crossterm::cursor::MoveToColumn((start + buffer.position() + 1) as u16));
    }
}

impl<'a> std::ops::Drop for Writer<'a> {
    fn drop(&mut self) {
        use std::io::Write;
        crossterm::terminal::disable_raw_mode();
        crossterm::execute!(
            self.lock,
            crossterm::style::ResetColor,
            crossterm::style::Print('\n')
        );
    }
}
