// TODO: Consider impact of using &str instead of &[char] in public API for completions

//! Provides completion methods for [`Prompt`] when reading lines.
//!
//! By default, no completions are performed upon user interaction. However, if a [`Completer`]
//! or a [`Suggester`] are provided, the [`Prompt`] will query for completions for the current
//! state of the line.
//!
//! [`Prompt`]: ../prompt/struct.Prompt.html
//! [`Completer`]: trait.Completer.html
//! [`Suggester`]: trait.Suggester.html

pub use crate::Context;

/// Completes the buffer in-line.
///
/// Whenever the line is edited, e.g. [`Write`] or [`Delete`], the [`Prompt`] will ask the
/// `Completer` for a possible completion to **append** to the current buffer.
///
/// The buffer is not actually changed, the completion is only rendered. A [`Complete`] action
/// must be issued to incorporate the completion into the buffer.
///
/// [`Prompt`]: ../prompt/struct.Prompt.html
/// [`Write`]: ../actions/enum.Action.html#variant.Write
/// [`Delete`]: ../actions/enum.Action.html#variant.Delete
/// [`Complete`]: ../actions/enum.Action.html#variant.Complete
pub trait Completer {
    /// Whenever the line is edited, e.g. [`Write`] or [`Delete`], the [`Prompt`] will ask the
    /// [`Completer`] for a possible completion to **append** to the current buffer.
    ///
    /// # Examples
    ///
    /// Basic implementation:
    ///
    /// ```no_run
    /// # struct Basic(Vec<Vec<char>>);
    /// # impl rucline::completion::Completer for Basic {
    /// fn complete_for(&self, context: &dyn rucline::Context) -> Option<&[char]> {
    ///     let buffer = context.buffer();
    ///     if buffer.is_empty() {
    ///         None
    ///     } else {
    ///         self.0
    ///             .iter()
    ///             .find(|completion| completion.starts_with(buffer))
    ///             .map(|completion| &completion[buffer.len()..])
    ///     }
    /// }
    /// # }
    /// ```
    ///
    /// **See also [`Basic`]**
    ///
    /// [`Completer`]: trait.Completer.html
    /// [`Basic`]: struct.Basic.html#implementations
    fn complete_for(&self, context: &dyn Context) -> Option<&[char]>;
}

/// Generates a list of possible values for the [`Prompt`] buffer, usually associated with the
/// Tab` key.
///
/// Whenever the [`Suggest`] action is triggered,  the [`Prompt`] will ask the
/// `Suggester` for a list of values to **replace** to the current buffer.
/// This list is kept by the [`Prompt`] for cycling back and forth until it is dropped by
/// either accepting the suggestions or cancelling it.
///
/// The buffer is not actually changed until the suggestion is accepted by either a [`Write`], a
/// [`Delete`], [`Accept`] or a [`Move`], while a suggestion is selected.
///
/// [`Prompt`]: ../prompt/struct.Prompt.html
/// [`Write`]: ../actions/enum.Action.html#variant.Write
/// [`Delete`]: ../actions/enum.Action.html#variant.Delete
/// [`Move`]: ../actions/enum.Action.html#variant.Move
/// [`Accept`]: ../actions/enum.Action.html#variant.Accept
/// [`Suggest`]: ../actions/enum.Action.html#variant.Suggest
pub trait Suggester {
    /// Whenever the [`Suggest`] action is triggered,  the [`Prompt`] will ask the
    /// `Suggester` for a list of values to **replace** to the current buffer.
    ///
    /// # Examples
    ///
    /// Basic implementation:
    ///
    /// ```no_run
    /// # struct Basic(Vec<Vec<char>>);
    /// # impl rucline::completion::Suggester for Basic {
    ///  fn suggest_for(&self, _: &dyn rucline::Context) -> Vec<&[char]> {
    ///     self.0.iter().map(Vec::as_slice).collect::<Vec<_>>()
    /// }
    /// # }
    /// ```
    ///
    /// **See also [`Basic`]**
    ///
    /// [`Completer`]: trait.Completer.html
    /// [`Basic`]: struct.Basic.html#implementations
    fn suggest_for(&self, context: &dyn Context) -> Vec<&[char]>;
}

pub struct Lambda<'a, F, R>
where
    F: Fn(&dyn Context) -> R,
{
    lambda: F,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a, F> std::convert::From<F> for Lambda<'a, F, Option<&'a [char]>>
where
    F: Fn(&dyn Context) -> Option<&'a [char]>,
{
    fn from(lambda: F) -> Self {
        Self {
            lambda,
            _phantom: std::marker::PhantomData {},
        }
    }
}

impl<'a, F> std::convert::From<F> for Lambda<'a, F, Vec<&'a [char]>>
where
    F: Fn(&dyn Context) -> Vec<&'a [char]>,
{
    fn from(lambda: F) -> Self {
        Self {
            lambda,
            _phantom: std::marker::PhantomData {},
        }
    }
}

impl<'a, F> Completer for Lambda<'a, F, Option<&'a [char]>>
where
    F: Fn(&dyn Context) -> Option<&'a [char]>,
{
    fn complete_for(&self, context: &dyn Context) -> Option<&[char]> {
        (self.lambda)(context)
    }
}

impl<'a, F> Suggester for Lambda<'a, F, Vec<&'a [char]>>
where
    F: Fn(&dyn Context) -> Vec<&'a [char]>,
{
    fn suggest_for(&self, context: &dyn Context) -> Vec<&[char]> {
        (self.lambda)(context)
    }
}

/// A basic implementation of a completion provider serving both as an example and as a useful
/// simple completer and suggester
pub struct Basic(Vec<Vec<char>>);

impl Basic {
    /// Creates a new instance from the list of `options` given
    ///
    /// # Example
    ///
    /// ```no_run
    /// use rucline::completion::Basic;
    ///
    /// let basic = Basic::new(&["some programmer was here", "some developer was there"]);
    /// ```
    ///
    /// # Arguments
    ///
    /// * `options` - A list of `&str` to serve as options for completion and suggestions.
    #[must_use]
    pub fn new(options: &[&str]) -> Self {
        Self(
            options
                .iter()
                .map(|string| string.chars().collect())
                .collect(),
        )
    }
}

impl Completer for Basic {
    // Allowed because it is more readable this way
    #[allow(clippy::find_map)]
    fn complete_for(&self, context: &dyn Context) -> Option<&[char]> {
        let buffer = context.buffer();
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
    fn suggest_for(&self, _: &dyn Context) -> Vec<&[char]> {
        self.0.iter().map(Vec::as_slice).collect::<Vec<_>>()
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
            let expected = ['3'];
            assert_eq!(basic.complete_for(&Mock::from("b")), Some(&expected[..]));
        }

        #[test]
        fn only_complete_the_remainder() {
            let basic = Basic::new(&["abcd", "abc"]);
            let expected = ['d'];
            assert_eq!(basic.complete_for(&Mock::from("abc")), Some(&expected[..]));
        }

        #[test]
        fn always_suggest() {
            let basic = Basic::new(&["a", "b", "c"]);
            let options = [['a'], ['b'], ['c']];
            let expected = vec![&options[0][..], &options[1][..], &options[2][..]];
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
            let expected = ['3'];
            assert_eq!(lambda.complete_for(&Mock::from("b")), Some(&expected[..]));
        }

        #[test]
        fn basic_lambda_suggester() {
            let basic = Basic::new(&["a", "b", "c"]);
            let lambda = Lambda::from(|c: &dyn Context| basic.suggest_for(c));
            let options = [['a'], ['b'], ['c']];
            let expected = vec![&options[0][..], &options[1][..], &options[2][..]];
            assert_eq!(&lambda.suggest_for(&Mock::empty()), &expected);
            assert_eq!(&lambda.suggest_for(&Mock::from("a")), &expected);
            assert_eq!(&lambda.suggest_for(&Mock::from("z")), &expected);
        }
    }
}
