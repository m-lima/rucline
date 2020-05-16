use crate::actions::{Direction, Range, Scope};

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

    /// Deletes the given [`scope`](../../actions/enum.Scope.html) from this buffer
    /// and updates the cursor accordingly.
    pub(super) fn delete(&mut self, scope: Scope) {
        use Direction::{Backward, Forward};
        use Range::{Line, Single, Word};
        use Scope::{Relative, WholeLine, WholeWord};

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

    /// Moves the cursor by [`range`](../../actions/enum.Range.html)
    pub(super) fn move_cursor(&mut self, range: Range, direction: Direction) {
        use Direction::{Backward, Forward};
        use Range::{Line, Single, Word};

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

impl std::convert::From<&[char]> for Buffer {
    fn from(string: &[char]) -> Self {
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

#[cfg(test)]
mod test {
    use super::{Buffer, Direction, Range, Scope};
    use crate::prompt::char_string::CharString;

    fn build_uut(string: &str) -> Buffer {
        Buffer {
            chars: CharString::from(string),
            cursor: 0,
        }
    }

    fn set_cursor(buffer: &mut Buffer, string: &str) {
        buffer.cursor = string.find('_').unwrap();
    }

    #[test]
    fn delete_char_forward() {
        let mut buffer = build_uut("asdf bas  as   v as  bas   asdf");

        // Delete from the middle
        set_cursor(&mut buffer, "asdf b_s  as   v as  bas   asdf");
        buffer.delete(Scope::Relative(Range::Single, Direction::Forward));
        assert_eq!(buffer.cursor, 6);
        assert_eq!(buffer.chars.to_string(), "asdf bs  as   v as  bas   asdf");

        // Delete from the end
        set_cursor(&mut buffer, "asdf bs  as   v as  bas   asd_");
        buffer.delete(Scope::Relative(Range::Single, Direction::Forward));
        assert_eq!(buffer.cursor, 29);
        assert_eq!(buffer.chars.to_string(), "asdf bs  as   v as  bas   asd");

        // Delete from past the end
        set_cursor(&mut buffer, "asdf bs  as   v as  bas   asd_");
        buffer.delete(Scope::Relative(Range::Single, Direction::Forward));
        assert_eq!(buffer.cursor, 29);
        assert_eq!(buffer.chars.to_string(), "asdf bs  as   v as  bas   asd");

        // Delete from the start
        set_cursor(&mut buffer, "_sdf bs  as   v as  bas   asd");
        buffer.delete(Scope::Relative(Range::Single, Direction::Forward));
        assert_eq!(buffer.cursor, 0);
        assert_eq!(buffer.chars.to_string(), "sdf bs  as   v as  bas   asd");
    }

    #[test]
    fn delete_char_backward() {
        let mut buffer = build_uut("asdf bas  as   v as  bas   asdf");

        // Delete from the middle
        set_cursor(&mut buffer, "asdf b_s  as   v as  bas   asdf");
        buffer.delete(Scope::Relative(Range::Single, Direction::Backward));
        assert_eq!(buffer.cursor, 5);
        assert_eq!(buffer.chars.to_string(), "asdf as  as   v as  bas   asdf");

        // Delete from the end
        set_cursor(&mut buffer, "asdf as  as   v as  bas   asd_");
        buffer.delete(Scope::Relative(Range::Single, Direction::Backward));
        assert_eq!(buffer.cursor, 28);
        assert_eq!(buffer.chars.to_string(), "asdf as  as   v as  bas   asf");

        // Delete from past the end
        set_cursor(&mut buffer, "asdf as  as   v as  bas   asf_");
        buffer.delete(Scope::Relative(Range::Single, Direction::Backward));
        assert_eq!(buffer.cursor, 28);
        assert_eq!(buffer.chars.to_string(), "asdf as  as   v as  bas   as");

        // Delete from the start
        set_cursor(&mut buffer, "_sdf as  as   v as  bas   as");
        buffer.delete(Scope::Relative(Range::Single, Direction::Backward));
        assert_eq!(buffer.cursor, 0);
        assert_eq!(buffer.chars.to_string(), "asdf as  as   v as  bas   as");
    }

    #[test]
    fn delete_word_forward() {
        let mut buffer = build_uut("asdf bas  as   v as  bas   asdf");

        // Delete from the middle
        set_cursor(&mut buffer, "as_f bas  as   v as  bas   asdf");
        buffer.delete(Scope::Relative(Range::Word, Direction::Forward));
        assert_eq!(buffer.cursor, 2);
        assert_eq!(buffer.chars.to_string(), "asbas  as   v as  bas   asdf");

        // Delete single letter word
        set_cursor(&mut buffer, "asbas  as   _ as  bas   asdf");
        buffer.delete(Scope::Relative(Range::Word, Direction::Forward));
        assert_eq!(buffer.cursor, 12);
        assert_eq!(buffer.chars.to_string(), "asbas  as   as  bas   asdf");

        // Delete from space
        set_cursor(&mut buffer, "asbas  as _ as  bas   asdf");
        buffer.delete(Scope::Relative(Range::Word, Direction::Forward));
        assert_eq!(buffer.cursor, 10);
        assert_eq!(buffer.chars.to_string(), "asbas  as as  bas   asdf");

        // Delete from the end
        set_cursor(&mut buffer, "asbas  as as  bas   asd_");
        buffer.delete(Scope::Relative(Range::Word, Direction::Forward));
        assert_eq!(buffer.cursor, 23);
        assert_eq!(buffer.chars.to_string(), "asbas  as as  bas   asd");

        // Delete from past the end
        set_cursor(&mut buffer, "asbas  as as  bas   asd_");
        buffer.delete(Scope::Relative(Range::Word, Direction::Forward));
        assert_eq!(buffer.cursor, 23);
        assert_eq!(buffer.chars.to_string(), "asbas  as as  bas   asd");

        // Delete from the start
        set_cursor(&mut buffer, "_sbas  as as  bas   asd");
        buffer.delete(Scope::Relative(Range::Word, Direction::Forward));
        assert_eq!(buffer.cursor, 0);
        assert_eq!(buffer.chars.to_string(), "as as  bas   asd");
    }

    #[test]
    fn delete_word_backward() {
        let mut buffer = build_uut("asdf bas  as   v as  bas   asdf");

        // Delete from the middle
        set_cursor(&mut buffer, "as_f bas  as   v as  bas   asdf");
        buffer.delete(Scope::Relative(Range::Word, Direction::Backward));
        assert_eq!(buffer.cursor, 0);
        assert_eq!(buffer.chars.to_string(), "df bas  as   v as  bas   asdf");

        // Delete single letter word
        set_cursor(&mut buffer, "df bas  as   _ as  bas   asdf");
        buffer.delete(Scope::Relative(Range::Word, Direction::Backward));
        assert_eq!(buffer.cursor, 8);
        assert_eq!(buffer.chars.to_string(), "df bas  v as  bas   asdf");

        // Delete from space
        set_cursor(&mut buffer, "df bas  v as  bas _ asdf");
        buffer.delete(Scope::Relative(Range::Word, Direction::Backward));
        assert_eq!(buffer.cursor, 14);
        assert_eq!(buffer.chars.to_string(), "df bas  v as    asdf");

        // Delete from the end
        set_cursor(&mut buffer, "df bas  v as    asd_");
        buffer.delete(Scope::Relative(Range::Word, Direction::Backward));
        assert_eq!(buffer.cursor, 16);
        assert_eq!(buffer.chars.to_string(), "df bas  v as    f");

        // Delete from past the end
        set_cursor(&mut buffer, "df bas  v as    f_");
        buffer.delete(Scope::Relative(Range::Word, Direction::Backward));
        assert_eq!(buffer.cursor, 16);
        assert_eq!(buffer.chars.to_string(), "df bas  v as    ");

        // Delete from the start
        set_cursor(&mut buffer, "_f bas  v as    ");
        buffer.delete(Scope::Relative(Range::Word, Direction::Backward));
        assert_eq!(buffer.cursor, 0);
        assert_eq!(buffer.chars.to_string(), "df bas  v as    ");
    }

    #[test]
    fn delete_line_forward() {
        let mut buffer = build_uut("asdf bas  as   v as  bas   asdf");

        // Delete from the middle
        set_cursor(&mut buffer, "asdf bas  as   _ as  bas   asdf");
        buffer.delete(Scope::Relative(Range::Line, Direction::Forward));
        assert_eq!(buffer.cursor, 15);
        assert_eq!(buffer.chars.to_string(), "asdf bas  as   ");

        // Delete from the end
        set_cursor(&mut buffer, "asdf bas  as   _");
        buffer.delete(Scope::Relative(Range::Line, Direction::Forward));
        assert_eq!(buffer.cursor, 15);
        assert_eq!(buffer.chars.to_string(), "asdf bas  as   ");

        // Delete from the start
        set_cursor(&mut buffer, "_sdf bas  as   ");
        buffer.delete(Scope::Relative(Range::Line, Direction::Forward));
        assert_eq!(buffer.cursor, 0);
        assert_eq!(buffer.chars.to_string(), "");

        // Delete empty line
        set_cursor(&mut buffer, "_");
        buffer.delete(Scope::Relative(Range::Line, Direction::Forward));
        assert_eq!(buffer.cursor, 0);
        assert_eq!(buffer.chars.to_string(), "");
    }

    #[test]
    fn delete_line_backward() {
        let mut buffer = build_uut("asdf bas  as   v as  bas   asdf");

        // Delete from the middle
        set_cursor(&mut buffer, "asdf bas  as   _ as  bas   asdf");
        buffer.delete(Scope::Relative(Range::Line, Direction::Backward));
        assert_eq!(buffer.cursor, 0);
        assert_eq!(buffer.chars.to_string(), "v as  bas   asdf");

        // Delete from the start
        set_cursor(&mut buffer, "_as  bas   asdf");
        buffer.delete(Scope::Relative(Range::Line, Direction::Backward));
        assert_eq!(buffer.cursor, 0);
        assert_eq!(buffer.chars.to_string(), "v as  bas   asdf");

        // Delete from the end
        set_cursor(&mut buffer, "v as  bas   asdf_");
        buffer.delete(Scope::Relative(Range::Line, Direction::Backward));
        assert_eq!(buffer.cursor, 0);
        assert_eq!(buffer.chars.to_string(), "");

        // Delete empty line
        set_cursor(&mut buffer, "_");
        buffer.delete(Scope::Relative(Range::Line, Direction::Backward));
        assert_eq!(buffer.cursor, 0);
        assert_eq!(buffer.chars.to_string(), "");
    }

    #[test]
    fn delete_whole_word() {
        let mut buffer = build_uut("asdf bas  as   v as  bas   asdf");

        // Delete from the middle
        set_cursor(&mut buffer, "as_f bas  as   v as  bas   asdf");
        buffer.delete(Scope::WholeWord);
        assert_eq!(buffer.cursor, 0);
        assert_eq!(buffer.chars.to_string(), "bas  as   v as  bas   asdf");

        // Delete single letter word
        set_cursor(&mut buffer, "bas  as   _ as  bas   asdf");
        buffer.delete(Scope::WholeWord);
        assert_eq!(buffer.cursor, 8);
        assert_eq!(buffer.chars.to_string(), "bas  as as  bas   asdf");

        // Delete from space
        set_cursor(&mut buffer, "bas  as as  bas _ asdf");
        buffer.delete(Scope::WholeWord);
        assert_eq!(buffer.cursor, 16);
        assert_eq!(buffer.chars.to_string(), "bas  as as  bas asdf");
    }

    #[test]
    fn delete_whole_line() {
        let mut buffer = build_uut("asdf bas  as   v as  bas   asdf");

        // Delete from the middle
        set_cursor(&mut buffer, "asdf bas  as   _ as  bas   asdf");
        buffer.delete(Scope::WholeLine);
        assert_eq!(buffer.cursor, 0);
        assert_eq!(buffer.chars.to_string(), "");

        // Delete empty line
        set_cursor(&mut buffer, "_");
        buffer.delete(Scope::WholeLine);
        assert_eq!(buffer.cursor, 0);
        assert_eq!(buffer.chars.to_string(), "");
    }
}
