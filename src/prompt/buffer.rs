use crate::key_bindings::{Direction, Range, Scope};

pub(super) struct Buffer {
    data: Vec<char>,
    position: usize,
}

impl Default for Buffer {
    fn default() -> Self {
        Self {
            data: Vec::new(),
            position: 0,
        }
    }
}

impl Buffer {
    pub(super) fn new() -> Self {
        Default::default()
    }

    pub(super) fn position(&self) -> &usize {
        &self.position
    }

    pub(super) fn at_end(&self) -> bool {
        self.position == self.data.len()
    }

    pub(super) fn data(&self) -> String {
        self.data.iter().collect()
    }

    pub(super) fn data_raw(&self) -> &Vec<char> {
        &self.data
    }

    pub(super) fn set_str(&mut self, string: &str) {
        self.clear();
        self.write_str(string);
    }

    pub(super) fn write_str(&mut self, string: &str) {
        self.data.extend(string.chars());
    }

    pub(super) fn clear(&mut self) {
        self.data.clear();
        self.position = 0;
    }

    pub(super) fn write(&mut self, c: char) {
        self.data.insert(self.position, c);
        self.position += 1;
    }

    pub(super) fn delete(&mut self, scope: Scope) {
        match scope {
            Scope::Relative(Range::Single(Direction::Backward)) => {
                if self.position > 0 {
                    self.data.remove(self.position - 1);
                    self.position -= 1;
                }
            }
            Scope::Relative(Range::Single(Direction::Forward)) => {
                if self.position < self.data.len() {
                    self.data.remove(self.position);
                }
            }
            Scope::Relative(Range::Word(Direction::Backward)) => {
                let index = super::navigation::previous_word(self.position, &self.data);
                self.data.drain(index..self.position);
                self.position = index;
            }
            Scope::Relative(Range::Word(Direction::Forward)) => {
                let index = super::navigation::next_word(self.position, &self.data);
                self.data.drain(self.position..index);
            }
            Scope::Relative(Range::Line(Direction::Backward)) => {
                self.data.drain(0..self.position);
                self.position = 0;
            }
            Scope::Relative(Range::Line(Direction::Forward)) => {
                self.data.drain(self.position..self.data.len());
            }
            Scope::WholeWord => {
                let start = super::navigation::previous_word_end(self.position, &self.data);
                let end = super::navigation::next_word(self.position, &self.data);
                self.data.drain(start + 1..end);
                self.data[start] = ' ';
                self.position = start;
            }
            Scope::WholeLina => self.clear(),
        }
    }

    pub(super) fn move_cursor(&mut self, movement: Range) {
        match movement {
            Range::Single(Direction::Backward) => {
                if self.position > 0 {
                    self.position -= 1;
                }
            }
            Range::Single(Direction::Forward) => {
                if self.position < self.data.len() {
                    self.position += 1;
                }
            }
            Range::Word(Direction::Backward) => {
                self.position = super::navigation::previous_word(self.position, &self.data);
            }
            Range::Word(Direction::Forward) => {
                self.position = super::navigation::next_word(self.position, &self.data);
            }
            Range::Line(Direction::Backward) => {
                self.position = 0;
            }
            Range::Line(Direction::Forward) => {
                if self.position < self.data.len() {
                    self.position = self.data.len();
                }
            }
        }
    }
}
