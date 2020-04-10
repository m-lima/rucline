pub(super) struct CharString(Vec<char>);

impl CharString {
    #[must_use]
    pub(super) fn new() -> Self {
        Self(Vec::new())
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
    pub(super) fn insert_str(&mut self, index: usize, other: &[char]) {
        if index == self.0.len() {
            self.0.extend_from_slice(other);
        } else {
            let tail = self.0.drain(index..).collect::<Vec<_>>();
            self.0.extend_from_slice(other);
            self.0.extend(tail);
        }
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

impl<I> std::ops::Index<I> for CharString
where
    I: std::slice::SliceIndex<[char]>,
{
    type Output = I::Output;
    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        &self.0[index]
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

impl std::convert::From<&[char]> for CharString {
    fn from(data: &[char]) -> Self {
        Self(data.iter().map(Clone::clone).collect())
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

#[derive(Copy, Clone)]
pub(super) struct CharStringView<'a>(&'a [char]);

impl<'a> std::convert::From<&'a [char]> for CharStringView<'a> {
    fn from(data: &'a [char]) -> Self {
        Self(data)
    }
}

impl std::ops::Deref for CharStringView<'_> {
    type Target = [char];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for CharStringView<'_> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;
        for c in self.0 {
            fmt.write_char(*c)?;
        }
        Ok(())
    }
}
