mod navigation;

use crate::actions::{Direction, Range, Scope};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Error {
    InvalidIndex,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "invalid cursor position")
    }
}

/// A `String` that also keeps track of its cursor position.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Buffer {
    string: String,
    cursor: usize,
}

impl Buffer {
    /// Creates an empty buffer.
    #[must_use]
    pub fn new() -> Self {
        Buffer::default()
    }

    /// Creates a new [`Buffer`] from `string` with a preset cursor position.
    ///
    /// This is a short-hand for:
    /// ```no_run
    /// # use rucline::buffer::{Buffer, Error};
    /// # fn with_cursor(string: String, cursor: usize) -> Result<(), Error> {
    /// let mut buffer = Buffer::from(string);
    /// buffer.set_cursor(cursor)
    /// # }
    /// ```
    ///
    /// # Errors
    /// * [`Error`] - If the curosr position does not fall into a character boundary.
    ///
    /// [`Error`]: enum.Error.html
    /// [`Buffer`]: struct.Buffer.html
    pub fn new_with_cursor<S: AsRef<str>>(string: S, cursor: usize) -> Result<Self, Error> {
        let mut buffer = Buffer::from(string);
        buffer.set_cursor(cursor).map(|_| buffer)
    }

    /// Returns the current buffer string.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.string
    }

    /// Returns the current position of the cursor.
    #[inline]
    #[must_use]
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Sets the cursor position in the [`buffer`].
    ///
    /// # Errors
    /// * [`Error`] - If the curosr position does not fall into a character boundary.
    ///
    /// [`Error`]: enum.Error.html
    /// [`buffer`]: struct.Buffer.html
    pub fn set_cursor(&mut self, cursor: usize) -> Result<(), Error> {
        if self.string.is_char_boundary(cursor) {
            self.cursor = cursor;
            Ok(())
        } else {
            Err(Error::InvalidIndex)
        }
    }

    /// Puts the cursor at the end of the buffer.
    ///
    /// This is short-hand for `move_cursor(Range::Line, Direction::Forward)`
    #[inline]
    pub fn go_to_end(&mut self) {
        self.cursor = self.string.len();
    }

    /// Clears the buffer and sets the cursor back to zero.
    #[inline]
    pub fn clear(&mut self) {
        self.string.clear();
        self.cursor = 0;
    }

    /// Inserts a single character to the buffer at the cursor position and increments
    /// the cursor by one.
    #[inline]
    pub fn write(&mut self, c: char) {
        self.string.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    /// Inserts a string to the buffer at the cursor position and increments
    /// the cursor by the length of `string`.
    #[inline]
    pub fn write_str(&mut self, string: &str) {
        self.string.insert_str(self.cursor, string);
        self.cursor += string.len();
    }

    /// Inserts a range of a string to the buffer at the cursor position and increments
    /// the cursor by the length of the range.
    #[inline]
    pub fn write_range(&mut self, string: &str, range: Range) {
        match range {
            Range::Line => {
                self.write_str(string);
            }
            Range::Word => {
                let index = navigation::next_word(0, &string);
                self.write_str(&string[0..index]);
            }
            Range::Single => {
                self.write(string.chars().next().unwrap());
            }
        }
    }

    /// Deletes the given [`scope`](../../actions/enum.Scope.html) from this buffer
    /// and updates the cursor accordingly.
    pub fn delete(&mut self, scope: Scope) {
        use Direction::{Backward, Forward};
        use Range::{Line, Single, Word};
        use Scope::{Relative, WholeLine, WholeWord};

        match scope {
            Relative(Single, Backward) => {
                if self.cursor > 0 {
                    if let Some((index, _)) = self.string[..self.cursor].char_indices().next_back()
                    {
                        self.cursor = index;
                        self.string.remove(self.cursor);
                    }
                }
            }
            Relative(Single, Forward) => {
                if self.cursor < self.string.len() {
                    self.string.remove(self.cursor);
                }
            }
            Relative(Word, Backward) => {
                let index = navigation::previous_word(self.cursor, &self.string);
                self.string.drain(index..self.cursor);
                self.cursor = index;
            }
            Relative(Word, Forward) => {
                let index = navigation::next_word(self.cursor, &self.string);
                self.string.drain(self.cursor..index);
            }
            Relative(Line, Backward) => {
                self.string.drain(0..self.cursor);
                self.cursor = 0;
            }
            Relative(Line, Forward) => {
                self.string.drain(self.cursor..self.string.len());
            }
            WholeWord => {
                let mut start = navigation::previous_word_end(self.cursor, &self.string);
                let mut end = navigation::next_word(self.cursor, &self.string);

                // If not in the start and there is white space at the boundary,
                // save one white space
                if start > 0 {
                    if let Some(c) = self.string[start..]
                        .chars()
                        .next()
                        .filter(|c| c.is_whitespace())
                    {
                        start += c.len_utf8();
                    } else if let Some(c) = self.string[..end]
                        .chars()
                        .next_back()
                        .filter(|c| c.is_whitespace())
                    {
                        end -= c.len_utf8();
                    }
                }

                self.string.drain(start..end);
                self.cursor = start;
            }
            WholeLine => self.clear(),
        }
    }

    /// Moves the cursor by [`range`](../../actions/enum.Range.html)
    pub fn move_cursor(&mut self, range: Range, direction: Direction) {
        use Direction::{Backward, Forward};
        use Range::{Line, Single, Word};

        match (range, direction) {
            (Single, Backward) => {
                self.cursor = navigation::previous_scalar_value(self.cursor, &self.string);
            }
            (Single, Forward) => {
                self.cursor = navigation::next_scalar_value(self.cursor, &self.string);
            }
            (Word, Backward) => {
                self.cursor = navigation::previous_word(self.cursor, &self.string);
            }
            (Word, Forward) => {
                self.cursor = navigation::next_word(self.cursor, &self.string);
            }
            (Line, Backward) => {
                self.cursor = 0;
            }
            (Line, Forward) => {
                if self.cursor < self.string.len() {
                    self.go_to_end();
                }
            }
        }
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self {
            string: String::new(),
            cursor: 0,
        }
    }
}

impl<S> std::convert::From<S> for Buffer
where
    S: AsRef<str>,
{
    fn from(string: S) -> Self {
        Self {
            string: String::from(string.as_ref()),
            cursor: string.as_ref().len(),
        }
    }
}

impl std::ops::Deref for Buffer {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.string
    }
}

impl std::fmt::Display for Buffer {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.string.fmt(fmt)
    }
}

// Allowed because it makes test clearer
#[allow(clippy::non_ascii_literal)]
#[cfg(test)]
mod test {
    use super::{Buffer, Direction, Range, Scope};

    const TEST_STRING: &str = "abcd \t e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq";

    #[test]
    fn write() {
        scenarios(
            |buffer: &mut Buffer| buffer.write('x'),
            Jig {
                empty: "x_",
                at_start: "x_abcd \t e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_single_char: "abcd \t x_e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                in_middle: "abcd \t e  fgx_hi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_end: "abcd \t e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷pqx_",
                in_space: "abcd x_\t e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_word_start: "abcd \t e  x_fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_word_end: "abcd \t e  fghx_i  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                before_emoji: "abcd \t e  fghi x_ 😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_emoji: "abcd \t e  fghi  x_😀  jk😀lm  🇧🇷  no🇧🇷pq",
                after_emoji: "abcd \t e  fghi  😀x_  jk😀lm  🇧🇷  no🇧🇷pq",
                until_emoji: "abcd \t e  fghi  😀  x_jk😀lm  🇧🇷  no🇧🇷pq",
                past_emoji: "abcd \t e  fghi  😀  jk😀lx_m  🇧🇷  no🇧🇷pq",
                before_flag: "abcd \t e  fghi  😀  jk😀lm x_ 🇧🇷  no🇧🇷pq",
                at_flag: "abcd \t e  fghi  😀  jk😀lm  x_🇧🇷  no🇧🇷pq",
                within_flag: "abcd \t e  fghi  😀  jk😀lm  🇧x_🇷  no🇧🇷pq",
                after_flag: "abcd \t e  fghi  😀  jk😀lm  🇧🇷x_  no🇧🇷pq",
                until_flag: "abcd \t e  fghi  😀  jk😀lm  🇧🇷  x_no🇧🇷pq",
                past_flag: "abcd \t e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷px_q",
            },
        );
    }

    #[test]
    fn write_large_unicode_scalar_value() {
        scenarios(
            |buffer: &mut Buffer| buffer.write('😎'),
            Jig {
                empty: "😎_",
                at_start: "😎_abcd \t e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_single_char: "abcd \t 😎_e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                in_middle: "abcd \t e  fg😎_hi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_end: "abcd \t e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq😎_",
                in_space: "abcd 😎_\t e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_word_start: "abcd \t e  😎_fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_word_end: "abcd \t e  fgh😎_i  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                before_emoji: "abcd \t e  fghi 😎_ 😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_emoji: "abcd \t e  fghi  😎_😀  jk😀lm  🇧🇷  no🇧🇷pq",
                after_emoji: "abcd \t e  fghi  😀😎_  jk😀lm  🇧🇷  no🇧🇷pq",
                until_emoji: "abcd \t e  fghi  😀  😎_jk😀lm  🇧🇷  no🇧🇷pq",
                past_emoji: "abcd \t e  fghi  😀  jk😀l😎_m  🇧🇷  no🇧🇷pq",
                before_flag: "abcd \t e  fghi  😀  jk😀lm 😎_ 🇧🇷  no🇧🇷pq",
                at_flag: "abcd \t e  fghi  😀  jk😀lm  😎_🇧🇷  no🇧🇷pq",
                within_flag: "abcd \t e  fghi  😀  jk😀lm  🇧😎_🇷  no🇧🇷pq",
                after_flag: "abcd \t e  fghi  😀  jk😀lm  🇧🇷😎_  no🇧🇷pq",
                until_flag: "abcd \t e  fghi  😀  jk😀lm  🇧🇷  😎_no🇧🇷pq",
                past_flag: "abcd \t e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷p😎_q",
            },
        );
    }

    #[test]
    fn partial_grapheme_cluster() {
        scenarios(
            |buffer: &mut Buffer| buffer.write('🈎'),
            Jig {
                empty: "🈎_",
                at_start: "🈎_abcd \t e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_single_char: "abcd \t 🈎_e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                in_middle: "abcd \t e  fg🈎_hi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_end: "abcd \t e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq🈎_",
                in_space: "abcd 🈎_\t e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_word_start: "abcd \t e  🈎_fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_word_end: "abcd \t e  fgh🈎_i  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                before_emoji: "abcd \t e  fghi 🈎_ 😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_emoji: "abcd \t e  fghi  🈎_😀  jk😀lm  🇧🇷  no🇧🇷pq",
                after_emoji: "abcd \t e  fghi  😀🈎_  jk😀lm  🇧🇷  no🇧🇷pq",
                until_emoji: "abcd \t e  fghi  😀  🈎_jk😀lm  🇧🇷  no🇧🇷pq",
                past_emoji: "abcd \t e  fghi  😀  jk😀l🈎_m  🇧🇷  no🇧🇷pq",
                before_flag: "abcd \t e  fghi  😀  jk😀lm 🈎_ 🇧🇷  no🇧🇷pq",
                at_flag: "abcd \t e  fghi  😀  jk😀lm  🈎_🇧🇷  no🇧🇷pq",
                within_flag: "abcd \t e  fghi  😀  jk😀lm  🇧🈎_🇷  no🇧🇷pq",
                after_flag: "abcd \t e  fghi  😀  jk😀lm  🇧🇷🈎_  no🇧🇷pq",
                until_flag: "abcd \t e  fghi  😀  jk😀lm  🇧🇷  🈎_no🇧🇷pq",
                past_flag: "abcd \t e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷p🈎_q",
            },
        );
    }

    #[test]
    fn write_str() {
        scenarios(
            |buffer: &mut Buffer| buffer.write_str("xyz"),
            Jig {
                empty: "xyz_",
                at_start: "xyz_abcd \t e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_single_char: "abcd \t xyz_e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                in_middle: "abcd \t e  fgxyz_hi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_end: "abcd \t e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷pqxyz_",
                in_space: "abcd xyz_\t e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_word_start: "abcd \t e  xyz_fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_word_end: "abcd \t e  fghxyz_i  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                before_emoji: "abcd \t e  fghi xyz_ 😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_emoji: "abcd \t e  fghi  xyz_😀  jk😀lm  🇧🇷  no🇧🇷pq",
                after_emoji: "abcd \t e  fghi  😀xyz_  jk😀lm  🇧🇷  no🇧🇷pq",
                until_emoji: "abcd \t e  fghi  😀  xyz_jk😀lm  🇧🇷  no🇧🇷pq",
                past_emoji: "abcd \t e  fghi  😀  jk😀lxyz_m  🇧🇷  no🇧🇷pq",
                before_flag: "abcd \t e  fghi  😀  jk😀lm xyz_ 🇧🇷  no🇧🇷pq",
                at_flag: "abcd \t e  fghi  😀  jk😀lm  xyz_🇧🇷  no🇧🇷pq",
                within_flag: "abcd \t e  fghi  😀  jk😀lm  🇧xyz_🇷  no🇧🇷pq",
                after_flag: "abcd \t e  fghi  😀  jk😀lm  🇧🇷xyz_  no🇧🇷pq",
                until_flag: "abcd \t e  fghi  😀  jk😀lm  🇧🇷  xyz_no🇧🇷pq",
                past_flag: "abcd \t e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷pxyz_q",
            },
        );
    }

    #[test]
    fn write_multiple_unicode_scalar_values() {
        scenarios(
            |buffer: &mut Buffer| buffer.write_str("🇳🇴"),
            Jig {
                empty: "🇳🇴_",
                at_start: "🇳🇴_abcd \t e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_single_char: "abcd \t 🇳🇴_e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                in_middle: "abcd \t e  fg🇳🇴_hi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_end: "abcd \t e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq🇳🇴_",
                in_space: "abcd 🇳🇴_\t e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_word_start: "abcd \t e  🇳🇴_fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_word_end: "abcd \t e  fgh🇳🇴_i  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                before_emoji: "abcd \t e  fghi 🇳🇴_ 😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_emoji: "abcd \t e  fghi  🇳🇴_😀  jk😀lm  🇧🇷  no🇧🇷pq",
                after_emoji: "abcd \t e  fghi  😀🇳🇴_  jk😀lm  🇧🇷  no🇧🇷pq",
                until_emoji: "abcd \t e  fghi  😀  🇳🇴_jk😀lm  🇧🇷  no🇧🇷pq",
                past_emoji: "abcd \t e  fghi  😀  jk😀l🇳🇴_m  🇧🇷  no🇧🇷pq",
                before_flag: "abcd \t e  fghi  😀  jk😀lm 🇳🇴_ 🇧🇷  no🇧🇷pq",
                at_flag: "abcd \t e  fghi  😀  jk😀lm  🇳🇴_🇧🇷  no🇧🇷pq",
                within_flag: "abcd \t e  fghi  😀  jk😀lm  🇧🇳🇴_🇷  no🇧🇷pq",
                after_flag: "abcd \t e  fghi  😀  jk😀lm  🇧🇷🇳🇴_  no🇧🇷pq",
                until_flag: "abcd \t e  fghi  😀  jk😀lm  🇧🇷  🇳🇴_no🇧🇷pq",
                past_flag: "abcd \t e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷p🇳🇴_q",
            },
        );
    }

    #[test]
    fn delete_char_backward() {
        scenarios(
            |buffer: &mut Buffer| {
                buffer.delete(Scope::Relative(Range::Single, Direction::Backward))
            },
            Jig {
                empty: "_",
                at_start: "_abcd \t e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_single_char: "abcd \t_e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                in_middle: "abcd \t e  f_hi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_end: "abcd \t e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷p_",
                in_space: "abcd_\t e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_word_start: "abcd \t e _fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_word_end: "abcd \t e  fg_i  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                before_emoji: "abcd \t e  fghi_ 😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_emoji: "abcd \t e  fghi _😀  jk😀lm  🇧🇷  no🇧🇷pq",
                after_emoji: "abcd \t e  fghi  _  jk😀lm  🇧🇷  no🇧🇷pq",
                until_emoji: "abcd \t e  fghi  😀 _jk😀lm  🇧🇷  no🇧🇷pq",
                past_emoji: "abcd \t e  fghi  😀  jk😀_m  🇧🇷  no🇧🇷pq",
                before_flag: "abcd \t e  fghi  😀  jk😀lm_ 🇧🇷  no🇧🇷pq",
                at_flag: "abcd \t e  fghi  😀  jk😀lm _🇧🇷  no🇧🇷pq",
                within_flag: "abcd \t e  fghi  😀  jk😀lm  _🇷  no🇧🇷pq",
                after_flag: "abcd \t e  fghi  😀  jk😀lm  🇧_  no🇧🇷pq",
                until_flag: "abcd \t e  fghi  😀  jk😀lm  🇧🇷 _no🇧🇷pq",
                past_flag: "abcd \t e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷_q",
            },
        );
    }

    #[test]
    fn delete_char_forward() {
        scenarios(
            |buffer: &mut Buffer| buffer.delete(Scope::Relative(Range::Single, Direction::Forward)),
            Jig {
                empty: "_",
                at_start: "_bcd \t e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_single_char: "abcd \t _  fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                in_middle: "abcd \t e  fg_i  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_end: "abcd \t e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq_",
                in_space: "abcd _ e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_word_start: "abcd \t e  _ghi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_word_end: "abcd \t e  fgh_  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                before_emoji: "abcd \t e  fghi _😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_emoji: "abcd \t e  fghi  _  jk😀lm  🇧🇷  no🇧🇷pq",
                after_emoji: "abcd \t e  fghi  😀_ jk😀lm  🇧🇷  no🇧🇷pq",
                until_emoji: "abcd \t e  fghi  😀  _k😀lm  🇧🇷  no🇧🇷pq",
                past_emoji: "abcd \t e  fghi  😀  jk😀l_  🇧🇷  no🇧🇷pq",
                before_flag: "abcd \t e  fghi  😀  jk😀lm _🇧🇷  no🇧🇷pq",
                at_flag: "abcd \t e  fghi  😀  jk😀lm  _🇷  no🇧🇷pq",
                within_flag: "abcd \t e  fghi  😀  jk😀lm  🇧_  no🇧🇷pq",
                after_flag: "abcd \t e  fghi  😀  jk😀lm  🇧🇷_ no🇧🇷pq",
                until_flag: "abcd \t e  fghi  😀  jk😀lm  🇧🇷  _o🇧🇷pq",
                past_flag: "abcd \t e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷p_",
            },
        );
    }

    #[test]
    fn delete_word_backward() {
        scenarios(
            |buffer: &mut Buffer| buffer.delete(Scope::Relative(Range::Word, Direction::Backward)),
            Jig {
                empty: "_",
                at_start: "_abcd \t e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_single_char: "_e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                in_middle: "abcd \t e  _hi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_end: "abcd \t e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷_",
                in_space: "_\t e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_word_start: "abcd \t _fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_word_end: "abcd \t e  _i  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                before_emoji: "abcd \t e  _ 😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_emoji: "abcd \t e  _😀  jk😀lm  🇧🇷  no🇧🇷pq",
                after_emoji: "abcd \t e  fghi  _  jk😀lm  🇧🇷  no🇧🇷pq",
                until_emoji: "abcd \t e  fghi  _jk😀lm  🇧🇷  no🇧🇷pq",
                past_emoji: "abcd \t e  fghi  😀  jk😀_m  🇧🇷  no🇧🇷pq",
                before_flag: "abcd \t e  fghi  😀  jk😀_ 🇧🇷  no🇧🇷pq",
                at_flag: "abcd \t e  fghi  😀  jk😀_🇧🇷  no🇧🇷pq",
                within_flag: "abcd \t e  fghi  😀  jk😀lm  _🇷  no🇧🇷pq",
                after_flag: "abcd \t e  fghi  😀  jk😀lm  _  no🇧🇷pq",
                until_flag: "abcd \t e  fghi  😀  jk😀lm  _no🇧🇷pq",
                past_flag: "abcd \t e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷_q",
            },
        );
    }

    #[test]
    fn delete_word_forward() {
        scenarios(
            |buffer: &mut Buffer| buffer.delete(Scope::Relative(Range::Word, Direction::Forward)),
            Jig {
                empty: "_",
                at_start: "_e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_single_char: "abcd \t _fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                in_middle: "abcd \t e  fg_😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_end: "abcd \t e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq_",
                in_space: "abcd _e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_word_start: "abcd \t e  _😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_word_end: "abcd \t e  fgh_😀  jk😀lm  🇧🇷  no🇧🇷pq",
                before_emoji: "abcd \t e  fghi _😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_emoji: "abcd \t e  fghi  _jk😀lm  🇧🇷  no🇧🇷pq",
                after_emoji: "abcd \t e  fghi  😀_jk😀lm  🇧🇷  no🇧🇷pq",
                until_emoji: "abcd \t e  fghi  😀  _😀lm  🇧🇷  no🇧🇷pq",
                past_emoji: "abcd \t e  fghi  😀  jk😀l_🇧🇷  no🇧🇷pq",
                before_flag: "abcd \t e  fghi  😀  jk😀lm _🇧🇷  no🇧🇷pq",
                at_flag: "abcd \t e  fghi  😀  jk😀lm  _no🇧🇷pq",
                within_flag: "abcd \t e  fghi  😀  jk😀lm  🇧_no🇧🇷pq",
                after_flag: "abcd \t e  fghi  😀  jk😀lm  🇧🇷_no🇧🇷pq",
                until_flag: "abcd \t e  fghi  😀  jk😀lm  🇧🇷  _🇧🇷pq",
                past_flag: "abcd \t e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷p_",
            },
        );
    }

    #[test]
    fn delete_line_backward() {
        scenarios(
            |buffer: &mut Buffer| buffer.delete(Scope::Relative(Range::Line, Direction::Backward)),
            Jig {
                empty: "_",
                at_start: "_abcd \t e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_single_char: "_e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                in_middle: "_hi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_end: "_",
                in_space: "_\t e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_word_start: "_fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_word_end: "_i  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                before_emoji: "_ 😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_emoji: "_😀  jk😀lm  🇧🇷  no🇧🇷pq",
                after_emoji: "_  jk😀lm  🇧🇷  no🇧🇷pq",
                until_emoji: "_jk😀lm  🇧🇷  no🇧🇷pq",
                past_emoji: "_m  🇧🇷  no🇧🇷pq",
                before_flag: "_ 🇧🇷  no🇧🇷pq",
                at_flag: "_🇧🇷  no🇧🇷pq",
                within_flag: "_🇷  no🇧🇷pq",
                after_flag: "_  no🇧🇷pq",
                until_flag: "_no🇧🇷pq",
                past_flag: "_q",
            },
        );
    }

    #[test]
    fn delete_line_forward() {
        scenarios(
            |buffer: &mut Buffer| buffer.delete(Scope::Relative(Range::Line, Direction::Forward)),
            Jig {
                empty: "_",
                at_start: "_",
                at_single_char: "abcd \t _",
                in_middle: "abcd \t e  fg_",
                at_end: "abcd \t e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq_",
                in_space: "abcd _",
                at_word_start: "abcd \t e  _",
                at_word_end: "abcd \t e  fgh_",
                before_emoji: "abcd \t e  fghi _",
                at_emoji: "abcd \t e  fghi  _",
                after_emoji: "abcd \t e  fghi  😀_",
                until_emoji: "abcd \t e  fghi  😀  _",
                past_emoji: "abcd \t e  fghi  😀  jk😀l_",
                before_flag: "abcd \t e  fghi  😀  jk😀lm _",
                at_flag: "abcd \t e  fghi  😀  jk😀lm  _",
                within_flag: "abcd \t e  fghi  😀  jk😀lm  🇧_",
                after_flag: "abcd \t e  fghi  😀  jk😀lm  🇧🇷_",
                until_flag: "abcd \t e  fghi  😀  jk😀lm  🇧🇷  _",
                past_flag: "abcd \t e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷p_",
            },
        );
    }

    #[test]
    fn delete_whole_word() {
        scenarios(
            |buffer: &mut Buffer| buffer.delete(Scope::WholeWord),
            Jig {
                empty: "_",
                at_start: "_e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_single_char: "abcd _fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                in_middle: "abcd \t e _😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_end: "abcd \t e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷_",
                in_space: "abcd _e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_word_start: "abcd \t e _😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_word_end: "abcd \t e _😀  jk😀lm  🇧🇷  no🇧🇷pq",
                before_emoji: "abcd \t e  fghi _😀  jk😀lm  🇧🇷  no🇧🇷pq",
                at_emoji: "abcd \t e  fghi _jk😀lm  🇧🇷  no🇧🇷pq",
                after_emoji: "abcd \t e  fghi _jk😀lm  🇧🇷  no🇧🇷pq",
                until_emoji: "abcd \t e  fghi  😀 _😀lm  🇧🇷  no🇧🇷pq",
                past_emoji: "abcd \t e  fghi  😀  jk😀_ 🇧🇷  no🇧🇷pq",
                before_flag: "abcd \t e  fghi  😀  jk😀lm _🇧🇷  no🇧🇷pq",
                at_flag: "abcd \t e  fghi  😀  jk😀lm _no🇧🇷pq",
                within_flag: "abcd \t e  fghi  😀  jk😀lm _no🇧🇷pq",
                after_flag: "abcd \t e  fghi  😀  jk😀lm _no🇧🇷pq",
                until_flag: "abcd \t e  fghi  😀  jk😀lm  🇧🇷 _🇧🇷pq",
                past_flag: "abcd \t e  fghi  😀  jk😀lm  🇧🇷  no🇧🇷_",
            },
        );
    }

    #[test]
    fn delete_whole_line() {
        scenarios(
            |buffer: &mut Buffer| buffer.delete(Scope::WholeLine),
            Jig {
                empty: "_",
                at_start: "_",
                at_single_char: "_",
                in_middle: "_",
                at_end: "_",
                in_space: "_",
                at_word_start: "_",
                at_word_end: "_",
                before_emoji: "_",
                at_emoji: "_",
                after_emoji: "_",
                until_emoji: "_",
                past_emoji: "_",
                before_flag: "_",
                at_flag: "_",
                within_flag: "_",
                after_flag: "_",
                until_flag: "_",
                past_flag: "_",
            },
        );
    }

    #[derive(Clone, Copy)]
    struct Jig {
        empty: &'static str,
        at_start: &'static str,
        at_single_char: &'static str,
        in_middle: &'static str,
        at_end: &'static str,
        in_space: &'static str,
        at_word_start: &'static str,
        at_word_end: &'static str,
        before_emoji: &'static str,
        at_emoji: &'static str,
        after_emoji: &'static str,
        until_emoji: &'static str,
        past_emoji: &'static str,
        before_flag: &'static str,
        at_flag: &'static str,
        within_flag: &'static str,
        after_flag: &'static str,
        until_flag: &'static str,
        past_flag: &'static str,
    }

    fn clean(string: &str) -> String {
        string.replace('_', "")
    }

    fn scenarios(action: impl Fn(&mut Buffer) + Copy, jig: Jig) {
        simple_positional_scenarios(action, jig);
        single_unicode_scalar_value_scenarios(action, jig);
        multiple_unicode_scalar_values_scenarios(action, jig);
    }

    fn simple_positional_scenarios(action: impl Fn(&mut Buffer), jig: Jig) {
        // Empty
        let mut buffer = Buffer::from("");
        action(&mut buffer);
        let mut cursor = jig.empty.find('_').expect("empty");
        assert_eq!(buffer.cursor, cursor, "empty");
        assert_eq!(buffer.string, clean(jig.empty), "empty");

        // Start
        let mut buffer = Buffer::from(TEST_STRING);
        buffer.cursor = 0;
        action(&mut buffer);
        cursor = jig.at_start.find('_').expect("at_start");
        assert_eq!(buffer.cursor, cursor, "at_start");
        assert_eq!(buffer.string, clean(jig.at_start), "at_start");

        // At single char
        let mut buffer = Buffer::from(TEST_STRING);
        buffer.cursor = TEST_STRING.find('e').unwrap();
        action(&mut buffer);
        cursor = jig.at_single_char.find('_').expect("at_single_char");
        assert_eq!(buffer.cursor, cursor, "at_single_char");
        assert_eq!(buffer.string, clean(jig.at_single_char), "at_single_char");

        // Middle
        let mut buffer = Buffer::from(TEST_STRING);
        buffer.cursor = TEST_STRING.find('h').unwrap();
        action(&mut buffer);
        cursor = jig.in_middle.find('_').expect("in_middle");
        assert_eq!(buffer.cursor, cursor, "in_middle");
        assert_eq!(buffer.string, clean(jig.in_middle), "in_middle");

        // End
        let mut buffer = Buffer::from(TEST_STRING);
        action(&mut buffer);
        cursor = jig.at_end.find('_').expect("at_end");
        assert_eq!(buffer.cursor, cursor, "at_end");
        assert_eq!(buffer.string, clean(jig.at_end), "at_end");

        // Space
        let mut buffer = Buffer::from(TEST_STRING);
        buffer.cursor = TEST_STRING.find('\t').unwrap();
        action(&mut buffer);
        cursor = jig.in_space.find('_').expect("in_space");
        assert_eq!(buffer.cursor, cursor, "in_space");
        assert_eq!(buffer.string, clean(jig.in_space), "in_space");

        // Word start
        let mut buffer = Buffer::from(TEST_STRING);
        buffer.cursor = TEST_STRING.find('f').unwrap();
        action(&mut buffer);
        cursor = jig.at_word_start.find('_').expect("at_word_start");
        assert_eq!(buffer.cursor, cursor, "at_word_start");
        assert_eq!(buffer.string, clean(jig.at_word_start), "at_word_start");

        // Word end
        let mut buffer = Buffer::from(TEST_STRING);
        buffer.cursor = TEST_STRING.find('i').unwrap();
        action(&mut buffer);
        cursor = jig.at_word_end.find('_').expect("at_word_end");
        assert_eq!(buffer.cursor, cursor, "at_word_end");
        assert_eq!(buffer.string, clean(jig.at_word_end), "at_word_end");
    }

    fn single_unicode_scalar_value_scenarios(action: impl Fn(&mut Buffer), jig: Jig) {
        // Before emoji
        let mut buffer = Buffer::from(TEST_STRING);
        buffer.cursor = TEST_STRING.find('😀').unwrap() - 1;
        action(&mut buffer);
        let mut cursor = jig.before_emoji.find('_').expect("before_emoji");
        assert_eq!(buffer.cursor, cursor, "before_emoji");
        assert_eq!(buffer.string, clean(jig.before_emoji), "before_emoji");

        // At emoji
        let mut buffer = Buffer::from(TEST_STRING);
        buffer.cursor = TEST_STRING.find('😀').unwrap();
        action(&mut buffer);
        cursor = jig.at_emoji.find('_').expect("at_emoji");
        assert_eq!(buffer.cursor, cursor, "at_emoji");
        assert_eq!(buffer.string, clean(jig.at_emoji), "at_emoji");

        // After emoji
        let mut buffer = Buffer::from(TEST_STRING);
        buffer.cursor = TEST_STRING.find('😀').unwrap() + '😀'.len_utf8();
        action(&mut buffer);
        cursor = jig.after_emoji.find('_').expect("after_emoji");
        assert_eq!(buffer.cursor, cursor, "after_emoji");
        assert_eq!(buffer.string, clean(jig.after_emoji), "after_emoji");

        // Until emoji
        let mut buffer = Buffer::from(TEST_STRING);
        buffer.cursor = TEST_STRING.find('j').unwrap();
        action(&mut buffer);
        cursor = jig.until_emoji.find('_').expect("until_emoji");
        assert_eq!(buffer.cursor, cursor, "until_emoji");
        assert_eq!(buffer.string, clean(jig.until_emoji), "until_emoji");

        // Past emoji
        let mut buffer = Buffer::from(TEST_STRING);
        buffer.cursor = TEST_STRING.find('m').unwrap();
        action(&mut buffer);
        cursor = jig.past_emoji.find('_').expect("past_emoji");
        assert_eq!(buffer.cursor, cursor, "past_emoji");
        assert_eq!(buffer.string, clean(jig.past_emoji), "past_emoji");
    }

    fn multiple_unicode_scalar_values_scenarios(action: impl Fn(&mut Buffer), jig: Jig) {
        // Before multiple unicode scalar values
        let mut buffer = Buffer::from(TEST_STRING);
        buffer.cursor = TEST_STRING.find("🇧🇷").unwrap() - 1;
        action(&mut buffer);
        let mut cursor = jig.before_flag.find('_').expect("before_flag");
        assert_eq!(buffer.cursor, cursor, "before_flag");
        assert_eq!(buffer.string, clean(jig.before_flag), "before_flag");

        // At multiple unicode scalar values
        let mut buffer = Buffer::from(TEST_STRING);
        buffer.cursor = TEST_STRING.find("🇧🇷").unwrap();
        action(&mut buffer);
        cursor = jig.at_flag.find('_').expect("at_flag");
        assert_eq!(buffer.cursor, cursor, "at_flag");
        assert_eq!(buffer.string, clean(jig.at_flag), "at_flag");

        // Within multiple unicode scalar values
        let mut buffer = Buffer::from(TEST_STRING);
        buffer.cursor = TEST_STRING.find("🇧🇷").unwrap() + 4;
        action(&mut buffer);
        cursor = jig.within_flag.find('_').expect("within_flag");
        assert_eq!(buffer.cursor, cursor, "within_flag");
        assert_eq!(buffer.string, clean(jig.within_flag), "within_flag");

        // After multiple unicode scalar values
        let mut buffer = Buffer::from(TEST_STRING);
        buffer.cursor = TEST_STRING.find("🇧🇷").unwrap() + "🇧🇷".len();
        action(&mut buffer);
        cursor = jig.after_flag.find('_').expect("after_flag");
        assert_eq!(buffer.cursor, cursor, "after_flag");
        assert_eq!(buffer.string, clean(jig.after_flag), "after_flag");

        // Until multiple unicode scalar values
        let mut buffer = Buffer::from(TEST_STRING);
        buffer.cursor = TEST_STRING.find('n').unwrap();
        action(&mut buffer);
        cursor = jig.until_flag.find('_').expect("until_flag");
        assert_eq!(buffer.cursor, cursor, "until_flag");
        assert_eq!(buffer.string, clean(jig.until_flag), "until_flag");

        // Past multiple unicode scalar values
        let mut buffer = Buffer::from(TEST_STRING);
        buffer.cursor = TEST_STRING.find('q').unwrap();
        action(&mut buffer);
        cursor = jig.past_flag.find('_').expect("past_flag");
        assert_eq!(buffer.cursor, cursor, "past_flag");
        assert_eq!(buffer.string, clean(jig.past_flag), "past_flag");
    }
}
