// TODO: Consider impact of using &str instead of &[char] in public API for completions
// TODO: Consider using lambdas instead of trait objects for completions

pub trait Completer {
    fn complete_for(&self, context: &[char]) -> Option<&[char]>;
}

pub trait Suggester {
    fn suggest_for(&self, context: &[char]) -> Vec<&[char]>;
}

pub struct Basic(Vec<Vec<char>>);

impl Basic {
    #[must_use]
    pub fn new(completions: &[&str]) -> Self {
        Self(
            completions
                .iter()
                .map(|string| string.chars().collect())
                .collect(),
        )
    }
}

impl Completer for Basic {
    // Allowed because it is more readable this way
    #[allow(clippy::find_map)]
    fn complete_for(&self, buffer: &[char]) -> Option<&[char]> {
        if buffer.is_empty() {
            None
        } else {
            self.0
                .iter()
                .find(|completion| completion.starts_with(buffer))
                .map(|completion| &completion[buffer.len()..])
        }
    }
}

impl Suggester for Basic {
    fn suggest_for(&self, _: &[char]) -> Vec<&[char]> {
        self.0.iter().map(Vec::as_slice).collect::<Vec<_>>()
    }
}
