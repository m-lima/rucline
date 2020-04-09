pub(super) struct CharString(Vec<char>);

impl Default for CharString {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl CharString {
    pub(super) fn new() -> Self {
        CharString::default()
    }

    #[inline]
    pub(super) fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub(super) fn insert(&mut self, index: usize, c: char) {
        self.0.insert(index, c);
    }

    #[inline]
    pub(super) fn clear(&mut self) {
        self.0.clear();
    }

    #[inline]
    pub(super) fn remove(&mut self, index: usize) {
        self.0.remove(index);
    }

    #[inline]
    pub(super) fn drain<R>(&mut self, range: R)
    where
        R: std::ops::RangeBounds<usize>,
    {
        self.0.drain(range);
    }
}

impl std::ops::Index<usize> for CharString {
    type Output = char;
    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl std::ops::IndexMut<usize> for CharString {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl std::ops::Deref for CharString {
    type Target = [char];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::convert::From<&str> for CharString {
    fn from(data: &str) -> Self {
        Self(data.chars().collect())
    }
}

impl std::convert::From<Vec<char>> for CharString {
    fn from(data: Vec<char>) -> Self {
        Self(data)
    }
}

impl std::fmt::Display for CharString {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;
        for c in &self.0 {
            fmt.write_char(*c)?;
        }
        Ok(())
    }
}
