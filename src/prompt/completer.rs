use super::CharString;

pub(super) struct Completer {
    // TODO: Store an iterator instead?
    completions: Vec<CharString>,
}

impl Completer {
    pub(super) fn complete_for(&self, buffer: &CharString) -> Option<&CharString> {
        if buffer.is_empty() {
            None
        } else {
            self.completions
                .iter()
                .find(|completion| completion.starts_with(buffer))
        }
    }
}

impl std::convert::From<&[&str]> for Completer {
    fn from(completions: &[&str]) -> Self {
        Self {
            completions: completions
                .iter()
                .map(std::ops::Deref::deref)
                .map(std::convert::Into::into)
                .collect(),
        }
    }
}
