use crate::actions::{Direction, Range, Scope};

/// A `String` that also keeps track of its cursor position.
pub(super) struct Buffer {
    string: String,
    cursor: usize,
}

impl Default for Buffer {
    fn default() -> Self {
        Self {
            string: String::new(),
            cursor: 0,
        }
    }
}

impl Buffer {
    /// Creates an empty buffer.
    pub(super) fn new() -> Self {
        Buffer::default()
    }

    /// Returns the buffer contents.
    #[inline]
    pub(super) fn as_str(&self) -> &str {
        &self.string
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
        self.cursor = self.string.len();
    }

    /// Clears the buffer and sets the cursor back to zero.
    #[inline]
    pub(super) fn clear(&mut self) {
        self.string.clear();
        self.cursor = 0;
    }

    /// Inserts a single character to the buffer at the cursor position and increments
    /// the cursor by one.
    #[inline]
    pub(super) fn write(&mut self, c: char) {
        self.string.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    /// Inserts a string to the buffer at the cursor position and increments
    /// the cursor by the length of `string`.
    #[inline]
    pub(super) fn write_str(&mut self, string: &str) {
        self.string.insert_str(self.cursor, string);
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
                let index = super::navigation::previous_word(self.cursor, &self.string);
                self.string.drain(index..self.cursor);
                self.cursor = index;
            }
            Relative(Word, Forward) => {
                let index = super::navigation::next_word(self.cursor, &self.string);
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
                let mut start = super::navigation::previous_word_end(self.cursor, &self.string);
                let end = super::navigation::next_word(self.cursor, &self.string);

                // If in the middle of the string, save one trailing space
                if start > 0 {
                    start += 1;
                }

                self.string.drain(start..end);
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
                self.cursor = super::navigation::previous_codepoint(self.cursor, &self.string);
            }
            (Single, Forward) => {
                self.cursor = super::navigation::next_codepoint(self.cursor, &self.string);
            }
            (Word, Backward) => {
                self.cursor = super::navigation::previous_word(self.cursor, &self.string);
            }
            (Word, Forward) => {
                self.cursor = super::navigation::next_word(self.cursor, &self.string);
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

impl std::convert::From<&str> for Buffer {
    fn from(string: &str) -> Self {
        Self {
            string: String::from(string),
            cursor: string.len(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Buffer, Direction, Range, Scope};

    fn build_uut(string: &str) -> Buffer {
        Buffer::from(string)
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
        assert_eq!(buffer.string.as_str(), "asdf bs  as   v as  bas   asdf");

        // Delete from the end
        set_cursor(&mut buffer, "asdf bs  as   v as  bas   asd_");
        buffer.delete(Scope::Relative(Range::Single, Direction::Forward));
        assert_eq!(buffer.cursor, 29);
        assert_eq!(buffer.string.as_str(), "asdf bs  as   v as  bas   asd");

        // Delete from past the end
        set_cursor(&mut buffer, "asdf bs  as   v as  bas   asd_");
        buffer.delete(Scope::Relative(Range::Single, Direction::Forward));
        assert_eq!(buffer.cursor, 29);
        assert_eq!(buffer.string.as_str(), "asdf bs  as   v as  bas   asd");

        // Delete from the start
        set_cursor(&mut buffer, "_sdf bs  as   v as  bas   asd");
        buffer.delete(Scope::Relative(Range::Single, Direction::Forward));
        assert_eq!(buffer.cursor, 0);
        assert_eq!(buffer.string.as_str(), "sdf bs  as   v as  bas   asd");
    }

    #[test]
    fn delete_char_backward() {
        let mut buffer = build_uut("asdf bas  as   v as  bas   asdf");

        // Delete from the middle
        set_cursor(&mut buffer, "asdf b_s  as   v as  bas   asdf");
        buffer.delete(Scope::Relative(Range::Single, Direction::Backward));
        assert_eq!(buffer.cursor, 5);
        assert_eq!(buffer.string.as_str(), "asdf as  as   v as  bas   asdf");

        // Delete from the end
        set_cursor(&mut buffer, "asdf as  as   v as  bas   asd_");
        buffer.delete(Scope::Relative(Range::Single, Direction::Backward));
        assert_eq!(buffer.cursor, 28);
        assert_eq!(buffer.string.as_str(), "asdf as  as   v as  bas   asf");

        // Delete from past the end
        set_cursor(&mut buffer, "asdf as  as   v as  bas   asf_");
        buffer.delete(Scope::Relative(Range::Single, Direction::Backward));
        assert_eq!(buffer.cursor, 28);
        assert_eq!(buffer.string.as_str(), "asdf as  as   v as  bas   as");

        // Delete from the start
        set_cursor(&mut buffer, "_sdf as  as   v as  bas   as");
        buffer.delete(Scope::Relative(Range::Single, Direction::Backward));
        assert_eq!(buffer.cursor, 0);
        assert_eq!(buffer.string.as_str(), "asdf as  as   v as  bas   as");
    }

    #[test]
    fn delete_word_forward() {
        let mut buffer = build_uut("asdf bas  as   v as  bas   asdf");

        // Delete from the middle
        set_cursor(&mut buffer, "as_f bas  as   v as  bas   asdf");
        buffer.delete(Scope::Relative(Range::Word, Direction::Forward));
        assert_eq!(buffer.cursor, 2);
        assert_eq!(buffer.string.as_str(), "asbas  as   v as  bas   asdf");

        // Delete single letter word
        set_cursor(&mut buffer, "asbas  as   _ as  bas   asdf");
        buffer.delete(Scope::Relative(Range::Word, Direction::Forward));
        assert_eq!(buffer.cursor, 12);
        assert_eq!(buffer.string.as_str(), "asbas  as   as  bas   asdf");

        // Delete from space
        set_cursor(&mut buffer, "asbas  as _ as  bas   asdf");
        buffer.delete(Scope::Relative(Range::Word, Direction::Forward));
        assert_eq!(buffer.cursor, 10);
        assert_eq!(buffer.string.as_str(), "asbas  as as  bas   asdf");

        // Delete from the end
        set_cursor(&mut buffer, "asbas  as as  bas   asd_");
        buffer.delete(Scope::Relative(Range::Word, Direction::Forward));
        assert_eq!(buffer.cursor, 23);
        assert_eq!(buffer.string.as_str(), "asbas  as as  bas   asd");

        // Delete from past the end
        set_cursor(&mut buffer, "asbas  as as  bas   asd_");
        buffer.delete(Scope::Relative(Range::Word, Direction::Forward));
        assert_eq!(buffer.cursor, 23);
        assert_eq!(buffer.string.as_str(), "asbas  as as  bas   asd");

        // Delete from the start
        set_cursor(&mut buffer, "_sbas  as as  bas   asd");
        buffer.delete(Scope::Relative(Range::Word, Direction::Forward));
        assert_eq!(buffer.cursor, 0);
        assert_eq!(buffer.string.as_str(), "as as  bas   asd");
    }

    #[test]
    fn delete_word_backward() {
        let mut buffer = build_uut("asdf bas  as   v as  bas   asdf");

        // Delete from the middle
        set_cursor(&mut buffer, "as_f bas  as   v as  bas   asdf");
        buffer.delete(Scope::Relative(Range::Word, Direction::Backward));
        assert_eq!(buffer.cursor, 0);
        assert_eq!(buffer.string.as_str(), "df bas  as   v as  bas   asdf");

        // Delete single letter word
        set_cursor(&mut buffer, "df bas  as   _ as  bas   asdf");
        buffer.delete(Scope::Relative(Range::Word, Direction::Backward));
        assert_eq!(buffer.cursor, 8);
        assert_eq!(buffer.string.as_str(), "df bas  v as  bas   asdf");

        // Delete from space
        set_cursor(&mut buffer, "df bas  v as  bas _ asdf");
        buffer.delete(Scope::Relative(Range::Word, Direction::Backward));
        assert_eq!(buffer.cursor, 14);
        assert_eq!(buffer.string.as_str(), "df bas  v as    asdf");

        // Delete from the end
        set_cursor(&mut buffer, "df bas  v as    asd_");
        buffer.delete(Scope::Relative(Range::Word, Direction::Backward));
        assert_eq!(buffer.cursor, 16);
        assert_eq!(buffer.string.as_str(), "df bas  v as    f");

        // Delete from past the end
        set_cursor(&mut buffer, "df bas  v as    f_");
        buffer.delete(Scope::Relative(Range::Word, Direction::Backward));
        assert_eq!(buffer.cursor, 16);
        assert_eq!(buffer.string.as_str(), "df bas  v as    ");

        // Delete from the start
        set_cursor(&mut buffer, "_f bas  v as    ");
        buffer.delete(Scope::Relative(Range::Word, Direction::Backward));
        assert_eq!(buffer.cursor, 0);
        assert_eq!(buffer.string.as_str(), "df bas  v as    ");
    }

    #[test]
    fn delete_line_forward() {
        let mut buffer = build_uut("asdf bas  as   v as  bas   asdf");

        // Delete from the middle
        set_cursor(&mut buffer, "asdf bas  as   _ as  bas   asdf");
        buffer.delete(Scope::Relative(Range::Line, Direction::Forward));
        assert_eq!(buffer.cursor, 15);
        assert_eq!(buffer.string.as_str(), "asdf bas  as   ");

        // Delete from the end
        set_cursor(&mut buffer, "asdf bas  as   _");
        buffer.delete(Scope::Relative(Range::Line, Direction::Forward));
        assert_eq!(buffer.cursor, 15);
        assert_eq!(buffer.string.as_str(), "asdf bas  as   ");

        // Delete from the start
        set_cursor(&mut buffer, "_sdf bas  as   ");
        buffer.delete(Scope::Relative(Range::Line, Direction::Forward));
        assert_eq!(buffer.cursor, 0);
        assert_eq!(buffer.string.as_str(), "");

        // Delete empty line
        set_cursor(&mut buffer, "_");
        buffer.delete(Scope::Relative(Range::Line, Direction::Forward));
        assert_eq!(buffer.cursor, 0);
        assert_eq!(buffer.string.as_str(), "");
    }

    #[test]
    fn delete_line_backward() {
        let mut buffer = build_uut("asdf bas  as   v as  bas   asdf");

        // Delete from the middle
        set_cursor(&mut buffer, "asdf bas  as   _ as  bas   asdf");
        buffer.delete(Scope::Relative(Range::Line, Direction::Backward));
        assert_eq!(buffer.cursor, 0);
        assert_eq!(buffer.string.as_str(), "v as  bas   asdf");

        // Delete from the start
        set_cursor(&mut buffer, "_as  bas   asdf");
        buffer.delete(Scope::Relative(Range::Line, Direction::Backward));
        assert_eq!(buffer.cursor, 0);
        assert_eq!(buffer.string.as_str(), "v as  bas   asdf");

        // Delete from the end
        set_cursor(&mut buffer, "v as  bas   asdf_");
        buffer.delete(Scope::Relative(Range::Line, Direction::Backward));
        assert_eq!(buffer.cursor, 0);
        assert_eq!(buffer.string.as_str(), "");

        // Delete empty line
        set_cursor(&mut buffer, "_");
        buffer.delete(Scope::Relative(Range::Line, Direction::Backward));
        assert_eq!(buffer.cursor, 0);
        assert_eq!(buffer.string.as_str(), "");
    }

    #[test]
    fn delete_whole_word() {
        let mut buffer = build_uut("asdf bas  as   v as  bas   asdf");

        // Delete from the middle
        set_cursor(&mut buffer, "as_f bas  as   v as  bas   asdf");
        buffer.delete(Scope::WholeWord);
        assert_eq!(buffer.cursor, 0);
        assert_eq!(buffer.string.as_str(), "bas  as   v as  bas   asdf");

        // Delete single letter word
        set_cursor(&mut buffer, "bas  as   _ as  bas   asdf");
        buffer.delete(Scope::WholeWord);
        assert_eq!(buffer.cursor, 8);
        assert_eq!(buffer.string.as_str(), "bas  as as  bas   asdf");

        // Delete from space
        set_cursor(&mut buffer, "bas  as as  bas _ asdf");
        buffer.delete(Scope::WholeWord);
        assert_eq!(buffer.cursor, 16);
        assert_eq!(buffer.string.as_str(), "bas  as as  bas asdf");
    }

    #[test]
    fn delete_whole_line() {
        let mut buffer = build_uut("asdf bas  as   v as  bas   asdf");

        // Delete from the middle
        set_cursor(&mut buffer, "asdf bas  as   _ as  bas   asdf");
        buffer.delete(Scope::WholeLine);
        assert_eq!(buffer.cursor, 0);
        assert_eq!(buffer.string.as_str(), "");

        // Delete empty line
        set_cursor(&mut buffer, "_");
        buffer.delete(Scope::WholeLine);
        assert_eq!(buffer.cursor, 0);
        assert_eq!(buffer.string.as_str(), "");
    }
}
