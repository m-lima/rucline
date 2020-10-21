//! Provides completion methods for [`prompt`] when reading lines.
//!
//! By default, no completions are performed upon user interaction. However, if a [`Completer`]
//! or a [`Suggester`] are provided, the [`prompt`] will query for completions for the current
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
//! * [`Actions`]
//!
//! [`Actions`]: ../actions/index.html
//! [`Completer`]: trait.Completer.html
//! [`Suggester`]: trait.Suggester.html
//! [`prompt`]: ../prompt/index.html

pub use crate::Buffer;

/// Completes the buffer in-line.
///
/// Whenever the line is edited, e.g. [`Write`] or [`Delete`], the [`prompt`] will ask the
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
/// * [`Suggester`]
///
/// [`Buffer`]: ../buffer/struct.Buffer.html
/// [`Complete`]: ../actions/enum.Action.html#variant.Complete
/// [`Delete`]: ../actions/enum.Action.html#variant.Delete
/// [`Suggester`]: trait.Suggester.html
/// [`Write`]: ../actions/enum.Action.html#variant.Write
/// [`prompt`]: ../prompt/index.html
pub trait Completer {
    /// Provides the in-line completion.
    ///
    /// Whenever the line is edited, e.g. [`Write`] or [`Delete`], the [`prompt`] will call
    /// `complete_for` for a possible completion to **append** to the current buffer.
    ///
    /// # Arguments
    /// * [`buffer`] - Read-only view into the line buffer, providing the context in which this
    /// event is happening.
    ///
    /// # Return
    /// * A completion to be rendered. `None` if there are no suggestions.
    ///
    /// # See also
    /// * [`Suggester`]
    ///
    /// [`Buffer`]: ../buffer/struct.Buffer.html
    /// [`Delete`]: ../actions/enum.Action.html#variant.Delete
    /// [`Suggester`]: trait.Suggester.html
    /// [`Write`]: ../actions/enum.Action.html#variant.Write
    /// [`prompt`]: ../prompt/index.html
    fn complete_for(&self, buffer: &Buffer) -> Option<std::borrow::Cow<'_, str>>;
}

/// Generates a list of possible values for the [`prompt`] buffer, usually associated with the
/// `Tab` key.
///
/// Whenever the [`Suggest`] action is triggered,  the [`prompt`] will ask the
/// `Suggester` for a list of values to **replace** to the current buffer.
/// This list is kept by the [`prompt`] for cycling back and forth until it is dropped by
/// either accepting a suggestion or canceling it. The implementation
/// may use the [`Buffer`] to decide which completions are applicable.
///
/// The buffer is not actually changed until the suggestion is accepted by either a [`Write`], a
/// [`Delete`], [`Accept`] or a [`Move`], while a suggestion is selected.
///
/// # See also
/// * [`Completer`]
///
/// [`Accept`]: ../actions/enum.Action.html#variant.Accept
/// [`Buffer`]: ../buffer/struct.Buffer.html
/// [`Completer`]: trait.Completer.html
/// [`Delete`]: ../actions/enum.Action.html#variant.Delete
/// [`Move`]: ../actions/enum.Action.html#variant.Move
/// [`Suggest`]: ../actions/enum.Action.html#variant.Suggest
/// [`Write`]: ../actions/enum.Action.html#variant.Write
/// [`prompt`]: ../prompt/index.html
pub trait Suggester {
    /// Whenever the [`Suggest`] action is triggered, the [`prompt`] will call `suggest_for`
    /// for a list of values to **replace** to the current buffer.
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
    /// * [`buffer`] - Read-only view into the line buffer, providing the context in which this
    /// event is happening.
    ///
    /// # Return
    /// * The list of suggestions to be rendered as drop-down options. Empty if none.
    ///
    /// [`Buffer`]: ../buffer/struct.Buffer.html
    /// [`Suggest`]: ../actions/enum.Action.html#variant.Suggest
    /// [`prompt`]: ../prompt/index.html
    fn suggest_for(&self, buffer: &Buffer) -> Vec<std::borrow::Cow<'_, str>>;
}

macro_rules! impl_completion {
    (completer) => {
        fn complete_for(&self, buffer: &Buffer) -> Option<std::borrow::Cow<'_, str>> {
            if buffer.is_empty() {
                None
            } else {
                self.iter().find_map(|completion| {
                    if completion.as_ref().starts_with(buffer.as_str()) {
                        Some((completion.as_ref()[buffer.len()..]).into())
                    } else {
                        None
                    }
                })
            }
        }
    };

    (suggester) => {
        fn suggest_for(&self, _: &Buffer) -> Vec<std::borrow::Cow<'_, str>> {
            self.iter()
                .map(|suggestion| suggestion.as_ref().into())
                .collect()
        }
    };
}

impl<S: AsRef<str>> Completer for Vec<S> {
    impl_completion!(completer);
}

impl<S: AsRef<str>> Completer for [S] {
    impl_completion!(completer);
}

impl<S: AsRef<str>> Completer for &[S] {
    impl_completion!(completer);
}

impl<S: AsRef<str>> Suggester for Vec<S> {
    impl_completion!(suggester);
}

impl<S: AsRef<str>> Suggester for [S] {
    impl_completion!(suggester);
}

impl<S: AsRef<str>> Suggester for &[S] {
    impl_completion!(suggester);
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
