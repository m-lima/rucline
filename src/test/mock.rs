pub struct Context {
    pub buffer: Vec<char>,
    pub cursor: usize,
}

impl crate::Context for Context {
    fn buffer(&self) -> &[char] {
        &self.buffer
    }

    fn cursor(&self) -> usize {
        self.cursor
    }
}

impl Context {
    pub fn empty() -> Self {
        Self {
            buffer: Vec::new(),
            cursor: 0,
        }
    }

    pub fn from(string: &str) -> Self {
        Self {
            buffer: string.chars().collect(),
            cursor: string.len(),
        }
    }
}
