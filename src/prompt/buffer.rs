use super::char_string::CharString;
use crate::key_bindings::{Direction, Range, Scope};

pub(super) struct Buffer {
    chars: CharString,
    position: usize,
}

impl Default for Buffer {
    fn default() -> Self {
        Self {
            chars: CharString::new(),
            position: 0,
        }
    }
}

impl Buffer {
    pub(super) fn new() -> Self {
        Buffer::default()
    }

    #[inline]
    pub(super) fn position(&self) -> usize {
        self.position
    }

    #[inline]
    pub(super) fn at_end(&self) -> bool {
        self.position == self.chars.len()
    }

    #[inline]
    pub(super) fn len(&self) -> usize {
        self.chars.len()
    }

    #[inline]
    pub(super) fn clear(&mut self) {
        self.chars.clear();
        self.position = 0;
    }

    #[inline]
    pub(super) fn write(&mut self, c: char) {
        self.chars.insert(self.position, c);
        self.position += 1;
    }

    pub(super) fn delete(&mut self, scope: Scope) {
        match scope {
            Scope::Relative(Range::Single(Direction::Backward)) => {
                if self.position > 0 {
                    self.chars.remove(self.position - 1);
                    self.position -= 1;
                }
            }
            Scope::Relative(Range::Single(Direction::Forward)) => {
                if self.position < self.chars.len() {
                    self.chars.remove(self.position);
                }
            }
            Scope::Relative(Range::Word(Direction::Backward)) => {
                let index = super::navigation::previous_word(self.position, &self.chars);
                self.chars.drain(index..self.position);
                self.position = index;
            }
            Scope::Relative(Range::Word(Direction::Forward)) => {
                let index = super::navigation::next_word(self.position, &self.chars);
                self.chars.drain(self.position..index);
            }
            Scope::Relative(Range::Line(Direction::Backward)) => {
                self.chars.drain(0..self.position);
                self.position = 0;
            }
            Scope::Relative(Range::Line(Direction::Forward)) => {
                self.chars.drain(self.position..self.chars.len());
            }
            Scope::WholeWord => {
                let start = super::navigation::previous_word_end(self.position, &self.chars);
                let end = super::navigation::next_word(self.position, &self.chars);
                self.chars.drain(start + 1..end);
                self.chars[start] = ' ';
                self.position = start;
            }
            Scope::WholeLine => self.clear(),
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
                if self.position < self.chars.len() {
                    self.position += 1;
                }
            }
            Range::Word(Direction::Backward) => {
                self.position = super::navigation::previous_word(self.position, &self.chars);
            }
            Range::Word(Direction::Forward) => {
                self.position = super::navigation::next_word(self.position, &self.chars);
            }
            Range::Line(Direction::Backward) => {
                self.position = 0;
            }
            Range::Line(Direction::Forward) => {
                if self.position < self.chars.len() {
                    self.position = self.chars.len();
                }
            }
        }
    }
}

impl std::fmt::Display for Buffer {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.chars.fmt(fmt)
    }
}
