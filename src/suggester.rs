pub trait Suggester {
    fn suggest_for(&self, buffer: &[char]) -> &[String];
}

pub struct Basic(Vec<String>);

impl Basic {
    #[must_use]
    pub fn new(completions: &[&str]) -> Self {
        Self(
            completions
                .iter()
                .map(|string| (*string).to_string())
                .collect(),
        )
    }
}

impl Suggester for Basic {
    fn suggest_for(&self, _: &[char]) -> &[String] {
        &self.0
    }
}
