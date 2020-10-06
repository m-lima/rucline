//! Provides a [`Context`] to pass into the customization hook calls.
//!
//! [`Context`]: trait.Context.html

/// The context of the prompt buffer.
///
/// This context is passed in to every hook call and represents the buffer and the current state
/// of the prompt.
pub trait Context {
    /// The current buffer of the prompt.
    ///
    /// That is, what has been written so far. It does not suggestions or completions.
    fn buffer(&self) -> &str;

    /// The current cursor position of the prompt. The position is relative to the context
    /// [`buffer`].
    ///
    /// [`buffer`]: trait.Context.html#tymethod.buffer
    fn cursor(&self) -> usize;
}
