//! Provides completion methods for [`Prompt`] when reading lines.
//!
//! By default, no completions are performed upon user interaction. However, if a [`Completer`]
//! or a [`Suggester`] are provided, the [`Prompt`] will query for completions for the current
//! state of the line.
//!
//! This module also includes a convenience wrapper for lambdas, allowing quick implementation
//! of completions with closures.
//!
//! # Examples
//!
//! Basic implementation for in-line completion:
//!
//! ```no_run
//! use rucline::completion::{Completer, Context};
//!
//! struct Basic(Vec<String>);
//! impl Completer for Basic {
//!   fn complete_for(&self, context: &dyn Context) -> Option<&str> {
//!       let buffer = context.buffer();
//!       if buffer.is_empty() {
//!           None
//!       } else {
//!           self.0
//!               .iter()
//!               .find(|completion| completion.starts_with(buffer))
//!               .map(|completion| &completion[buffer.len()..])
//!       }
//!   }
//! }
//! ```
//!
//! Basic implementation for drop-down suggestions:
//!
//! ```no_run
//! use rucline::completion::{Context, Suggester};
//!
//! struct Basic(Vec<String>);
//! impl Suggester for Basic {
//!   fn suggest_for(&self, context: &dyn Context) -> Vec<&str> {
//!       self.0.iter().map(String::as_str).collect()
//!   }
//! }
//! ```
//!
//! Basic implementation for drop-down suggestions with lambda:
//!
//! ```no_run
//! use rucline::completion::{Context, Lambda};
//!
//! let completions = ["abc", "def", "example word", "weird \"˚∆˙\""];
//! let completer = Lambda::from(|c: &dyn Context| {
//!    completions
//!        .iter()
//!        .filter_map(|s| if s.starts_with(c.buffer()) { Some(*s) } else { None })
//!        .collect::<Vec<_>>()
//!});
//! ```
//!
//! # See also
//! * [`Basic`]
//! * [`Lambda`]
//!
//! [`Basic`]: struct.Basic.html
//! [`Lambda`]: struct.Lambda.html
//! [`Prompt`]: ../prompt/struct.Prompt.html
//! [`Completer`]: trait.Completer.html
//! [`Suggester`]: trait.Suggester.html

pub use crate::Buffer;

/// Completes the buffer in-line.
///
/// Whenever the line is edited, e.g. [`Write`] or [`Delete`], the [`Prompt`] will ask the
/// `Completer` for a possible completion to **append** to the current buffer. The implementation
/// may use the [`Context`] to decide which completions are applicable.
///
/// The buffer is not actually changed, the completion is only rendered. A [`Complete`] action
/// must be issued to incorporate the completion into the buffer.
///
/// # Example
///
/// Basic implementation:
///
/// ```no_run
/// use rucline::completion::{Completer, Context};
///
/// struct Basic(Vec<String>);
/// impl Completer for Basic {
///   fn complete_for(&self, context: &dyn Context) -> Option<&str> {
///       let buffer = context.buffer();
///       if buffer.is_empty() {
///           None
///       } else {
///           self.0
///               .iter()
///               .find(|completion| completion.starts_with(buffer))
///               .map(|completion| &completion[buffer.len()..])
///       }
///   }
/// }
/// ```
///
/// # See also
/// * [`Basic`]
///
/// [`Basic`]: struct.Basic.html#implementations
/// [`Complete`]: ../actions/enum.Action.html#variant.Complete
/// [`Context`]: ../prompt/context/trait.Context.html
/// [`Delete`]: ../actions/enum.Action.html#variant.Delete
/// [`Prompt`]: ../prompt/struct.Prompt.html
/// [`Write`]: ../actions/enum.Action.html#variant.Write
pub trait Completer {
    /// Provides the in-line completion for a given [`Context`].
    ///
    /// Whenever the line is edited, e.g. [`Write`] or [`Delete`], the [`Prompt`] will call
    /// `complete_for` for a possible completion to **append** to the current buffer.
    ///
    /// # Arguments
    /// * [`context`] - The current context in which this event is coming in.
    ///
    /// # Return
    /// * [`Option<&str>`] - A completion to be rendered. `None` if there are no suggestions.
    ///
    /// # See also
    /// * [`Basic`]
    ///
    /// [`Context`]: ../prompt/context/trait.Context.html
    /// [`Completer`]: trait.Completer.html
    /// [`Basic`]: struct.Basic.html#implementations
    fn complete_for(&self, buffer: &Buffer) -> Option<std::borrow::Cow<'_, str>>;
}

/// Generates a list of possible values for the [`Prompt`] buffer, usually associated with the
/// `Tab` key.
///
/// Whenever the [`Suggest`] action is triggered,  the [`Prompt`] will ask the
/// `Suggester` for a list of values to **replace** to the current buffer.
/// This list is kept by the [`Prompt`] for cycling back and forth until it is dropped by
/// either accepting the suggestions or cancelling it. The implementation
/// may use the [`Context`] to decide which completions are applicable.
///
/// The buffer is not actually changed until the suggestion is accepted by either a [`Write`], a
/// [`Delete`], [`Accept`] or a [`Move`], while a suggestion is selected.
///
/// [`Accept`]: ../actions/enum.Action.html#variant.Accept
/// [`Context`]: ../prompt/context/trait.Context.html
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
    /// # struct Basic(Vec<String>);
    /// # impl rucline::completion::Suggester for Basic {
    ///  fn suggest_for(&self, _: &dyn rucline::Context) -> Vec<&str> {
    ///     self.0.iter().map(String::as_str).collect()
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

impl<F> Completer for F
where
    F: Fn(&Buffer) -> Option<std::borrow::Cow<'static, str>>,
{
    fn complete_for(&self, buffer: &Buffer) -> Option<std::borrow::Cow<'_, str>> {
        self(buffer)
    }
}

impl<F> Suggester for F
where
    F: Fn(&Buffer) -> Vec<std::borrow::Cow<'static, str>>,
{
    fn suggest_for(&self, buffer: &Buffer) -> Vec<std::borrow::Cow<'_, str>> {
        self(buffer)
    }
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
    mod basic {
        use super::super::{Basic, Completer, Suggester};
        use crate::test::mock::Context as Mock;

        #[test]
        fn should_not_complete_if_empty() {
            let basic = Basic::new(&["some programmer was here", "some developer was there"]);
            assert_eq!(basic.complete_for(&Mock::empty()), None);
        }

        #[test]
        fn should_not_complete_if_context_is_different() {
            let basic = Basic::new(&["some programmer was here", "some developer was there"]);
            assert_eq!(basic.complete_for(&Mock::from("a")), None);
        }

        #[test]
        fn complete_the_first_match() {
            let basic = Basic::new(&["zz", "b3", "b2"]);
            let expected = "3";
            assert_eq!(basic.complete_for(&Mock::from("b")), Some(expected));
        }

        #[test]
        fn only_complete_the_remainder() {
            let basic = Basic::new(&["abcd", "abc"]);
            let expected = "d";
            assert_eq!(basic.complete_for(&Mock::from("abc")), Some(expected));
        }

        #[test]
        fn always_suggest() {
            let basic = Basic::new(&["a", "b", "c"]);
            let expected = vec!["a", "b", "c"];
            assert_eq!(&basic.suggest_for(&Mock::empty()), &expected);
            assert_eq!(&basic.suggest_for(&Mock::from("a")), &expected);
            assert_eq!(&basic.suggest_for(&Mock::from("z")), &expected);
        }
    }

    mod lambda {
        use super::super::{Basic, Completer, Context, Lambda, Suggester};
        use crate::test::mock::Context as Mock;

        #[test]
        fn lambdas_can_bu_used_for_both_completions() {
            let lambda = Lambda::from(|_: &dyn Context| None);
            assert_eq!(lambda.complete_for(&Mock::empty()), None);

            let lambda = Lambda::from(|_: &dyn Context| vec![]);
            assert!(lambda.suggest_for(&Mock::empty()).is_empty());
        }

        #[test]
        fn basic_lambda_completer() {
            let basic = Basic::new(&["zz", "b3", "b2"]);
            let lambda = Lambda::from(|c: &dyn Context| basic.complete_for(c));
            let expected = "3";
            assert_eq!(lambda.complete_for(&Mock::from("b")), Some(expected));
        }

        #[test]
        fn basic_lambda_suggester() {
            let basic = Basic::new(&["a", "b", "c"]);
            let lambda = Lambda::from(|c: &dyn Context| basic.suggest_for(c));
            let expected = vec!["a", "b", "c"];
            assert_eq!(&lambda.suggest_for(&Mock::empty()), &expected);
            assert_eq!(&lambda.suggest_for(&Mock::from("a")), &expected);
            assert_eq!(&lambda.suggest_for(&Mock::from("z")), &expected);
        }
    }
}
