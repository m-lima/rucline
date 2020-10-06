pub struct Context {
    pub buffer: String,
    pub cursor: usize,
}

impl crate::Context for Context {
    fn buffer(&self) -> &str {
        &self.buffer
    }

    fn cursor(&self) -> usize {
        self.cursor
    }
}

impl Context {
    pub fn empty() -> Self {
        Self {
            buffer: String::new(),
            cursor: 0,
        }
    }

    pub fn from(string: &str) -> Self {
        Self {
            buffer: String::from(string),
            cursor: string.len(),
        }
    }
}
