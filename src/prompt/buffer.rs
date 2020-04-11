use crate::key_bindings::{Direction, Range, Scope};

use super::CharString;

/// A [`CharString`](../char_string/struct.CharString.html) that also keeps track of its
/// cursor position.
///
/// Every editing method indirectly calls the underlying [`CharString`](../char_string/struct.CharString.html)
/// and updates the cursor position.
pub(super) struct Buffer {
    chars: CharString,
    cursor: usize,
}

impl Default for Buffer {
    fn default() -> Self {
        Self {
            chars: CharString::new(),
            cursor: 0,
        }
    }
}

impl Buffer {
    /// Creates an empty buffer.
    pub(super) fn new() -> Self {
        Buffer::default()
    }

    /// Returns the current position of the cursor.
    #[inline]
    pub(super) fn cursor(&self) -> usize {
        self.cursor
    }

    /// Returns if the current position of the cursor is at the end of the buffer.
    #[inline]
    pub(super) fn at_end(&self) -> bool {
        self.cursor == self.chars.len()
    }

    /// Puts the cursor at the end of the buffer
    /// This is short-hand for `move_cursor(Range::Line, Direction::Forward)`
    #[inline]
    pub(super) fn go_to_end(&mut self) {
        self.cursor = self.chars.len();
    }

    /// Returns the length of the buffer.
    #[inline]
    pub(super) fn len(&self) -> usize {
        self.chars.len()
    }

    /// Clears the buffer and sets the cursor back to zero.
    #[inline]
    pub(super) fn clear(&mut self) {
        self.chars.clear();
        self.cursor = 0;
    }

    /// Inserts a single character to the buffer at the cursor position and increments
    /// the cursor by one.
    #[inline]
    pub(super) fn write(&mut self, c: char) {
        self.chars.insert(self.cursor, c);
        self.cursor += 1;
    }

    /// Inserts a slice of characters to the buffer at the cursor position and increments
    /// the cursor by the length of `string`.
    #[inline]
    pub(super) fn write_str(&mut self, string: &[char]) {
        self.chars.insert_str(self.cursor, string);
        self.cursor += string.len();
    }

    /// Deletes the given [`scope`](../../key_bindings/enum.Scope.html) from this buffer
    /// and updates the cursor accordingly.
    pub(super) fn delete(&mut self, scope: Scope) {
        use Direction::*;
        use Range::*;
        use Scope::*;

        match scope {
            Relative(Single, Backward) => {
                if self.cursor > 0 {
                    self.chars.remove(self.cursor - 1);
                    self.cursor -= 1;
                }
            }
            Relative(Single, Forward) => {
                if self.cursor < self.chars.len() {
                    self.chars.remove(self.cursor);
                }
            }
            Relative(Word, Backward) => {
                let index = super::navigation::previous_word(self.cursor, &self.chars);
                self.chars.drain(index..self.cursor);
                self.cursor = index;
            }
            Relative(Word, Forward) => {
                let index = super::navigation::next_word(self.cursor, &self.chars);
                self.chars.drain(self.cursor..index);
            }
            Relative(Line, Backward) => {
                self.chars.drain(0..self.cursor);
                self.cursor = 0;
            }
            Relative(Line, Forward) => {
                self.chars.drain(self.cursor..self.chars.len());
            }
            WholeWord => {
                let mut start = super::navigation::previous_word_end(self.cursor, &self.chars);
                let end = super::navigation::next_word(self.cursor, &self.chars);

                // If in the middle of the string, save one trailing space
                if start > 0 {
                    start += 1;
                }

                self.chars.drain(start..end);
                self.cursor = start;
            }
            WholeLine => self.clear(),
        }
    }

    /// Moves the cursor by [`range`](../../key_bindings/enum.Range.html)
    pub(super) fn move_cursor(&mut self, range: Range, direction: Direction) {
        use Direction::*;
        use Range::*;

        match (range, direction) {
            (Single, Backward) => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
            }
            (Single, Forward) => {
                if self.cursor < self.chars.len() {
                    self.cursor += 1;
                }
            }
            (Word, Backward) => {
                self.cursor = super::navigation::previous_word(self.cursor, &self.chars);
            }
            (Word, Forward) => {
                self.cursor = super::navigation::next_word(self.cursor, &self.chars);
            }
            (Line, Backward) => {
                self.cursor = 0;
            }
            (Line, Forward) => {
                if self.cursor < self.chars.len() {
                    self.cursor = self.chars.len();
                }
            }
        }
    }
}

impl std::ops::Deref for Buffer {
    type Target = CharString;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.chars
    }
}

impl std::convert::From<&str> for Buffer {
    fn from(string: &str) -> Self {
        let chars = CharString::from(string);
        let cursor = chars.len();
        Self { chars, cursor }
    }
}

impl std::fmt::Display for Buffer {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.chars.fmt(fmt)
    }
}
