//! Provides completion methods for [`Prompt`] when reading lines.
//!
//! By default, no completions are performed upon user interaction. However, if a [`Completer`]
//! or a [`Suggester`] are provided, the [`Prompt`] will query for completions for the current
//! state of the line.
//!
//! This module also includes a convenience wrapper for lists, allowing quick implementation
//! of completions.
//!
//! # Examples
//!
//! Basic implementation for in-line completion:
//!
//! ```no_run
//! use rucline::Buffer;
//! use rucline::completion::Completer;
//! use std::borrow::Cow;
//!
//! struct Basic(Vec<String>);
//! impl Completer for Basic {
//!   fn complete_for(&self, buffer: &Buffer) -> Option<Cow<'_, str>> {
//!       if buffer.is_empty() {
//!           None
//!       } else {
//!           self.0
//!               .iter()
//!               .find(|completion| completion.starts_with(buffer.as_str()))
//!               .map(|completion| (&completion[buffer.len()..]).into())
//!       }
//!   }
//! }
//! ```
//!
//! Basic implementation for drop-down suggestions:
//!
//! ```no_run
//! use rucline::Buffer;
//! use rucline::completion::Suggester;
//! use std::borrow::Cow;
//!
//! struct Basic(Vec<String>);
//! impl Suggester for Basic {
//!   fn suggest_for(&self, buffer: &Buffer) -> Vec<Cow<'_, str>> {
//!       self.0.iter().map(|suggestion| suggestion.into()).collect()
//!   }
//! }
//! ```
//!
//! Basic implementation for drop-down suggestions with list:
//!
//! ```no_run
//! use rucline::completion::Completer;
//!
//! let completions = vec!["abc", "def", "example word", "weird \"˚∆˙\""];
//! let completer: &dyn Completer = &completions;
//! ```
//!
//! # See also
//! * [`Basic`]
//! * [`Lambda`]
//!
//! [`Prompt`]: ../prompt/struct.Prompt.html
//! [`Completer`]: trait.Completer.html
//! [`Suggester`]: trait.Suggester.html

pub use crate::Buffer;

/// Completes the buffer in-line.
///
/// Whenever the line is edited, e.g. [`Write`] or [`Delete`], the [`Prompt`] will ask the
/// `Completer` for a possible completion to **append** to the current buffer. The implementation
/// may use the [`Buffer`] to decide which completions are applicable.
///
/// When the `Completer` is invoked, the buffer is not actually changed, the completion is
/// only rendered. A [`Complete`] action must be issued to incorporate the completion into
/// the buffer.
///
/// # Example
///
/// Basic implementation:
///
/// ```no_run
/// use rucline::Buffer;
/// use rucline::completion::Completer;
/// use std::borrow::Cow;
///
/// struct Basic(Vec<String>);
/// impl Completer for Basic {
///   fn complete_for(&self, buffer: &Buffer) -> Option<Cow<'_, str>> {
///       if buffer.is_empty() {
///           None
///       } else {
///           self.0
///               .iter()
///               .find(|completion| completion.starts_with(buffer.as_str()))
///               .map(|completion| (&completion[buffer.len()..]).into())
///       }
///   }
/// }
/// ```
///
/// # See also
/// * [`Basic`]
///
/// [`Complete`]: ../actions/enum.Action.html#variant.Complete
/// [`Buffer`]: ../buffer/struct.Buffer.html
/// [`Delete`]: ../actions/enum.Action.html#variant.Delete
/// [`Prompt`]: ../prompt/struct.Prompt.html
/// [`Write`]: ../actions/enum.Action.html#variant.Write
pub trait Completer {
    /// Provides the in-line completion for a given [`Buffer`].
    ///
    /// Whenever the line is edited, e.g. [`Write`] or [`Delete`], the [`Prompt`] will call
    /// `complete_for` for a possible completion to **append** to the current buffer.
    ///
    /// # Arguments
    /// * [`buffer`] - The current context in which this event is coming in.
    ///
    /// # Return
    /// * [`Option<Cow<'_, str>>`] - A completion to be rendered. `None` if there are
    /// no suggestions.
    ///
    /// # See also
    /// * [`Suggester`]
    ///
    /// [`Buffer`]: ../buffer/struct.Buffer.html
    /// [`Completer`]: trait.Completer.html
    /// [`Suggester`]: trait.Suggester.html
    fn complete_for(&self, buffer: &Buffer) -> Option<std::borrow::Cow<'_, str>>;
}

/// Generates a list of possible values for the [`Prompt`] buffer, usually associated with the
/// `Tab` key.
///
/// Whenever the [`Suggest`] action is triggered,  the [`Prompt`] will ask the
/// `Suggester` for a list of values to **replace** to the current buffer.
/// This list is kept by the [`Prompt`] for cycling back and forth until it is dropped by
/// either accepting a suggestion or cancelling it. The implementation
/// may use the [`Buffer`] to decide which completions are applicable.
///
/// The buffer is not actually changed until the suggestion is accepted by either a [`Write`], a
/// [`Delete`], [`Accept`] or a [`Move`], while a suggestion is selected.
///
/// [`Accept`]: ../actions/enum.Action.html#variant.Accept
/// [`Buffer`]: ../buffer/struct.Buffer.html
/// [`Delete`]: ../actions/enum.Action.html#variant.Delete
/// [`Move`]: ../actions/enum.Action.html#variant.Move
/// [`Prompt`]: ../prompt/struct.Prompt.html
/// [`Suggest`]: ../actions/enum.Action.html#variant.Suggest
/// [`Write`]: ../actions/enum.Action.html#variant.Write
pub trait Suggester {
    /// Whenever the [`Suggest`] action is triggered,  the [`Prompt`] will ask the
    /// `Suggester` for a list of values to **replace** to the current buffer.
    ///
    /// # Examples
    ///
    /// Basic implementation:
    ///
    /// ```no_run
    /// # use std::borrow::Cow;
    /// # use rucline::Buffer;
    /// # struct Basic(Vec<String>);
    /// # impl rucline::completion::Suggester for Basic {
    ///  fn suggest_for(&self, _: &Buffer) -> Vec<Cow<'_, str>> {
    ///     self.0.iter().map(Into::into).collect()
    /// }
    /// # }
    /// ```
    ///
    /// # Arguments
    /// * [`context`] - The current context in which this event is coming in.
    ///
    /// # Return
    /// * [`Vec<&str>`] - The suggestions to be rendered as drop-down options. Empty if none.
    ///
    /// # See also
    /// * [`Basic`]
    ///
    /// [`Basic`]: struct.Basic.html#implementations
    /// [`Completer`]: trait.Completer.html
    /// [`Context`]: ../prompt/context/trait.Context.html
    fn suggest_for(&self, buffer: &Buffer) -> Vec<std::borrow::Cow<'_, str>>;
}

impl<S: AsRef<str>> Completer for Vec<S> {
    fn complete_for(&self, buffer: &Buffer) -> Option<std::borrow::Cow<'_, str>> {
        if buffer.is_empty() {
            None
        } else {
            self.iter()
                .find(|completion| completion.as_ref().starts_with(buffer.as_str()))
                .map(|completion| (completion.as_ref()[buffer.len()..]).into())
        }
    }
}

impl<S: AsRef<str>> Completer for [S] {
    fn complete_for(&self, buffer: &Buffer) -> Option<std::borrow::Cow<'_, str>> {
        if buffer.is_empty() {
            None
        } else {
            self.iter()
                .find(|completion| completion.as_ref().starts_with(buffer.as_str()))
                .map(|completion| (completion.as_ref()[buffer.len()..]).into())
        }
    }
}

impl<S: AsRef<str>> Suggester for Vec<S> {
    fn suggest_for(&self, _: &Buffer) -> Vec<std::borrow::Cow<'_, str>> {
        self.iter()
            .map(|suggestion| suggestion.as_ref().into())
            .collect()
    }
}

impl<S: AsRef<str>> Suggester for [S] {
    fn suggest_for(&self, _: &Buffer) -> Vec<std::borrow::Cow<'_, str>> {
        self.iter()
            .map(|suggestion| suggestion.as_ref().into())
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::{Buffer, Completer, Suggester};
    use std::borrow::Cow;

    #[test]
    fn should_not_complete_if_empty() {
        let list = ["some programmer was here", "some developer was there"];
        assert_eq!(list.complete_for(&Buffer::new()), None);
    }

    #[test]
    fn should_not_complete_if_context_is_different() {
        let list = ["some programmer was here", "some developer was there"];
        assert_eq!(list.complete_for(&"a".into()), None);
    }

    #[test]
    fn complete_the_first_match() {
        let list = ["zz", "b3", "b2"];
        let expected = Cow::Borrowed("3");
        assert_eq!(list.complete_for(&"b".into()), Some(expected));
    }

    #[test]
    fn only_complete_the_remainder() {
        let list = ["abcd", "abc"];
        let expected = Cow::Borrowed("d");
        assert_eq!(list.complete_for(&"abc".into()), Some(expected));
    }

    #[test]
    fn always_suggest() {
        let list = ["a", "b", "c"];
        let expected = vec!["a", "b", "c"];
        assert_eq!(&list.suggest_for(&Buffer::new()), &expected);
        assert_eq!(&list.suggest_for(&"a".into()), &expected);
        assert_eq!(&list.suggest_for(&"z".into()), &expected);
    }
}
