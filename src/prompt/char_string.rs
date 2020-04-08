pub(super) struct CharString {
    pub(super) data: Vec<char>,
}

impl Default for CharString {
    fn default() -> Self {
        Self { data: Vec::new() }
    }
}

impl CharString {
    pub(super) fn new() -> Self {
        Default::default()
    }
}

impl std::convert::From<&str> for CharString {
    fn from(data: &str) -> Self {
        Self {
            data: data.chars().collect(),
        }
    }
}

impl std::fmt::Display for CharString {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;
        for c in self.data.iter() {
            fmt.write_char(*c)?;
        }
        Ok(())
    }
}
