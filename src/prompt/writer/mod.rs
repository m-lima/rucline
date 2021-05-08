use super::Buffer;

pub(super) mod crossterm;

/// Implement this to provide your own display style.
pub trait Writer {
    type Error;

    /// Print the prompt, leaving the cursor in position to receive user input.
    fn begin(&mut self, prompt: Option<&str>) -> Result<(), Self::Error>;
    /// Print the user input, followed by the completion (if any).
    fn print(&mut self, buffer: &Buffer, completion: Option<&str>) -> Result<(), Self::Error>;
    /// Print the list of suggestions.
    fn print_suggestions(
        &mut self,
        selected_index: usize,
        suggestions: &[std::borrow::Cow<'_, str>],
    ) -> Result<(), Self::Error>;
}
