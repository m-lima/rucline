use super::Outcome;
use super::{RuclineWriter, Writer};

use crate::actions::{Action, Event, Overrider};
use crate::completion::{Completer, Suggester};
use crate::Buffer;
use crate::Error;

macro_rules! impl_builder {
    (base) => {
        fn buffer(mut self, buffer: Buffer) -> Self {
            self.base = self.base.buffer(buffer);
            self
        }

        fn erase_after_read(mut self, erase_after_read: bool) -> Self {
            self.base = self.base.erase_after_read(erase_after_read);
            self
        }
    };

    (extensions) => {
        fn overrider<O: Overrider>(self, overrider: O) -> WithOverrider<O, Self> {
            WithOverrider {
                base: self,
                overrider,
            }
        }

        fn overrider_fn<O>(self, overrider: O) -> WithOverrider<O, Self>
        where
            O: Fn($crate::actions::Event, &Buffer) -> Option<$crate::actions::Action>,
        {
            WithOverrider {
                base: self,
                overrider,
            }
        }

        fn overrider_ref<O: Overrider>(self, overrider: &O) -> WithRefOverrider<'_, O, Self> {
            WithRefOverrider {
                base: self,
                overrider,
            }
        }

        fn completer<C: Completer>(self, completer: C) -> WithCompleter<C, Self> {
            WithCompleter {
                base: self,
                completer,
            }
        }

        fn completer_fn<'a, F, R>(
            self,
            closure: F,
        ) -> WithCompleter<Closure<'a, F, Option<R>>, Self>
        where
            F: Fn(&Buffer) -> Option<R>,
            R: Into<std::borrow::Cow<'a, str>>,
        {
            WithCompleter {
                base: self,
                completer: Closure {
                    closure,
                    _phantom: std::marker::PhantomData,
                },
            }
        }

        fn completer_ref<C: Completer>(self, completer: &C) -> WithRefCompleter<'_, C, Self> {
            WithRefCompleter {
                base: self,
                completer,
            }
        }

        fn suggester<S: Suggester>(self, suggester: S) -> WithSuggester<S, Self> {
            WithSuggester {
                base: self,
                suggester,
            }
        }

        fn suggester_fn<'a, F, R>(self, closure: F) -> WithSuggester<Closure<'a, F, Vec<R>>, Self>
        where
            F: Fn(&Buffer) -> Vec<R>,
            R: Into<std::borrow::Cow<'a, str>>,
        {
            WithSuggester {
                base: self,
                suggester: Closure {
                    closure,
                    _phantom: std::marker::PhantomData,
                },
            }
        }

        fn suggester_ref<S: Suggester>(self, suggester: &S) -> WithRefSuggester<'_, S, Self> {
            WithRefSuggester {
                base: self,
                suggester,
            }
        }
    };
}

/// Builder for a line reader, providing methods that allows chaining together parameters for
/// customizing the behavior of the line reader.
///
/// It essentially is a helper for crafting an invocation of [`prompt::read_line`].
///
/// # Example
///
/// ```no_run
/// use rucline::prompt::{Builder, Prompt};
///
/// let outcome = Prompt::from("Delete file? ")
///     .completer(vec!["yes", "no"])
///     .erase_after_read(true)
///     .read_line();
/// ```
///
/// # Re-using the prompt configuration
///
/// The builder is consumed on every method call, including the [`read_line`] method. To re-use a
/// prompt, it is advisable to create a prompt provider.
/// For instance:
///
/// ```no_run
/// # fn do_something(r: Result<rucline::Outcome, rucline::Error>) {}
/// use rucline::prompt::{Builder, Prompt};
///
/// let completions = vec!["some", "completions"];
/// let suggestions = vec!["some", "suggestions"];
///
/// let prompt = |text: &str| Prompt::from(text)
///     .completer_ref(&completions)
///     .suggester_ref(&suggestions);
///
/// let firstOutcome = prompt("First: ").read_line();
/// do_something(firstOutcome);
///
/// let secondOutcome = prompt("Second: ").read_line();
/// do_something(secondOutcome);
/// ```
///
/// # Adding multiple hooks of the same kind
///
/// Setting two hooks to the same event in the same chain will cause only the last hook to be called.
/// For instance:
/// ```no_run
/// use rucline::prompt::{Builder, Prompt};
///
/// let some_completions = vec!["yes", "no"];
/// let some_other_completions = vec!["ja", "nei"];
///
/// let prompt = Prompt::from("Delete file? ")
///     .completer(some_completions)         // Will be ignored
///     .completer(some_other_completions);  // Superseeds the previous completer
/// ```
///
/// [`prompt::read_line`]: fn.read_line.html
/// [`read_line`]: trait.Builder.html#tymethod.read_line.html
pub trait Builder: ChainedLineReader + Sized {
    /// Prepopulates the prompt input with `buffer`.
    ///
    /// # Arguments
    /// * [`buffer`] - A buffer to be used when displaying the prompt.
    ///
    /// [`buffer`]: ../buffer/struct.Buffer.html
    fn buffer(self, buffer: Buffer) -> Self;

    /// Controls if the prompt shall be erased after user input.
    ///
    /// If set to `false` (default), after user input, the terminal will receive a new line
    /// after the prompt text and the user input. Any drop-down completions will be removed,
    /// however.
    ///
    /// If set to `true`, the whole prompt and input will be erased. The cursor returns to the
    /// original position as if nothing happened.
    ///
    /// # Arguments
    /// * `erase_after_read` - Whether the prompt shall be erased after user input.
    fn erase_after_read(self, erase_after_read: bool) -> Self;

    /// Modifies the behavior of the prompt by setting an [`Overrider`].
    ///
    /// The builder will take ownership of [`overrider`]. To pass in a reference, use
    /// [`overrider_ref`].
    ///
    /// # Arguments
    /// * [`overrider`] - The new overrider.
    ///
    /// [`overrider`]: ../actions/trait.Overrider.html
    /// [`overrider_ref`]: trait.Builder.html#tymethod.overrider_ref
    fn overrider<O: Overrider>(self, overrider: O) -> WithOverrider<O, Self>;

    /// Modifies the behavior of the prompt by setting an [`Overrider`] closure.
    ///
    /// # Arguments
    /// * [`overrider`] - The new overrider.
    ///
    /// [`Overrider`]: ../actions/trait.Overrider.html
    fn overrider_fn<O>(self, overrider: O) -> WithOverrider<O, Self>
    where
        O: Fn(Event, &Buffer) -> Option<Action>;

    /// Modifies the behavior of the prompt by setting a [`Overrider`] reference.
    ///
    /// # Arguments
    /// * [`overrider`] - The new overrider reference.
    ///
    /// [`Overrider`]: ../actions/trait.Overrider.html
    fn overrider_ref<O: Overrider>(self, overrider: &O) -> WithRefOverrider<'_, O, Self>;

    /// Sets the in-line completion provider.
    ///
    /// The builder will take ownership of [`completer`]. To pass in a reference, use
    /// [`completer_ref`].
    ///
    /// # Arguments
    /// * [`completer`] - The new completer.
    ///
    /// [`completer`]: ../completion/trait.Completer.html
    /// [`completer_ref`]: trait.Builder.html#tymethod.completer_ref
    fn completer<C: Completer>(self, completer: C) -> WithCompleter<C, Self>;

    /// Sets the in-line completion closure.
    ///
    /// # Arguments
    /// * [`completer`] - A closure that provides an optional completion.
    ///
    /// [`Completer`]: ../completion/trait.Completer.html
    fn completer_fn<'a, F, R>(self, closure: F) -> WithCompleter<Closure<'a, F, Option<R>>, Self>
    where
        F: Fn(&Buffer) -> Option<R>,
        R: Into<std::borrow::Cow<'a, str>>;

    /// Sets the in-line completion provider reference.
    ///
    /// # Arguments
    /// * [`completer`] - The new completer referece.
    ///
    /// [`Completer`]: ../completion/trait.Completer.html
    fn completer_ref<C: Completer>(self, completer: &C) -> WithRefCompleter<'_, C, Self>;

    /// Sets the drop-down suggestion provider.
    ///
    /// The builder will take ownership of [`suggester`]. To pass in a reference, use
    /// [`suggester_ref`].
    ///
    /// # Arguments
    /// * [`suggester`] - The new suggester.
    ///
    /// [`Suggester`]: ../completion/trait.Suggester.html
    /// [`suggester_ref`]: trait.Builder.html#tymethod.suggester_ref
    fn suggester<S: Suggester>(self, suggester: S) -> WithSuggester<S, Self>;

    /// Sets the drop-down suggestion closure.
    ///
    /// # Arguments
    /// * [`suggester`] - A closure that provides a list of suggestions.
    ///
    /// [`Suggester`]: ../completion/trait.Suggester.html
    fn suggester_fn<'a, F, R>(self, closure: F) -> WithSuggester<Closure<'a, F, Vec<R>>, Self>
    where
        F: Fn(&Buffer) -> Vec<R>,
        R: Into<std::borrow::Cow<'a, str>>;

    /// Sets the drop-down suggestion provider reference.
    ///
    /// # Arguments
    ///
    /// * [`suggester`] - The new suggester reference.
    ///
    /// [`Suggester`]: ../completion/trait.Suggester.html
    fn suggester_ref<S: Suggester>(self, suggester: &S) -> WithRefSuggester<'_, S, Self>;

    /// Consumes this [`Builder`] to craft an invocation of [`prompt::read_line`].
    ///
    /// # Errors
    /// * [`Error`] - If an error occurred while reading the user input.
    ///
    /// [`Builder`]: trait.Builder.html
    /// [`prompt::read_line`]: fn.read_line.html
    /// [`Error`]: ../enum.Error.html
    fn read_line(self) -> Result<Outcome, Error>;
}

pub trait ChainedLineReader {
    fn chain_read_line<O, C, S>(
        self,
        overrider: Option<&O>,
        completer: Option<&C>,
        suggester: Option<&S>,
    ) -> Result<Outcome, Error>
    where
        O: Overrider + ?Sized,
        C: Completer + ?Sized,
        S: Suggester + ?Sized;
}

/// The base struct for building a line reader prompt.
///
/// This is the most basic implementation of a [`Builder`], containing only a prompt text and a buffer.
/// It can however can be extended by using its [`Builder`] trait implementation. For example:
///
/// ```no_run
/// use rucline::prompt::{Builder, Prompt};
///
/// let prompt = Prompt::from("Type name: ")
///     .buffer("rust".into())
///     .suggester(vec!["rust", "c", "c++", "go"]);
/// ```
///
/// [`Builder`]: trait.Builder.html
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Prompt<W> {
    buffer: Option<Buffer>,
    writer: Option<W>,
    prompt: Option<String>,
    erase_after_read: bool,
}

impl<W: Writer> Prompt<W> {
    /// Creates a new [`Prompt`] with no prompt text. Equivalent to calling `Prompt::default()`
    ///
    /// [`Prompt`]: struct.Prompt.html
    #[must_use]
    pub fn new() -> Self {
        Self {
            writer: None,
            prompt: None,
            erase_after_read: false,
            buffer: None,
        }
    }
}

impl<W: Writer> Default for Prompt<W> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: ToString> std::convert::From<S> for Prompt<RuclineWriter<'_>> {
    fn from(s: S) -> Self {
        Self {
            buffer: None,
            writer: None,
            prompt: Some(s.to_string()),
            erase_after_read: false,
        }
    }
}

impl<W: Writer> std::convert::From<W> for Prompt<W> {
    fn from(w: W) -> Self {
        Self {
            buffer: None,
            writer: Some(w),
            prompt: None,
            erase_after_read: false,
        }
    }
}

pub struct WithOverrider<O, B>
where
    O: Overrider,
    B: Builder,
{
    base: B,
    overrider: O,
}

pub struct WithCompleter<C, B>
where
    C: Completer,
    B: Builder,
{
    base: B,
    completer: C,
}

pub struct WithSuggester<S, B>
where
    S: Suggester,
    B: Builder,
{
    base: B,
    suggester: S,
}

pub struct WithRefOverrider<'o, O, B>
where
    O: Overrider + ?Sized,
    B: Builder,
{
    base: B,
    overrider: &'o O,
}

pub struct WithRefCompleter<'c, C, B>
where
    C: Completer + ?Sized,
    B: Builder,
{
    base: B,
    completer: &'c C,
}

pub struct WithRefSuggester<'s, S, B>
where
    S: Suggester + ?Sized,
    B: Builder,
{
    base: B,
    suggester: &'s S,
}

impl Builder for Prompt<RuclineWriter<'_>> {
    fn read_line(self) -> Result<Outcome, Error> {
        let mut writer = RuclineWriter::new(self.erase_after_read, self.prompt.as_deref());
        super::read_line::<Dummy, Dummy, Dummy, RuclineWriter<'_>>(
            self.buffer,
            None,
            None,
            None,
            &mut writer,
        )
    }
}

impl<W: Writer> Builder for Prompt<W> {
    fn buffer(mut self, buffer: Buffer) -> Self {
        self.buffer = Some(buffer);
        self
    }

    fn erase_after_read(mut self, erase_after_read: bool) -> Self {
        self.erase_after_read = erase_after_read;
        self
    }

    impl_builder!(extensions);

    default fn read_line(self) -> Result<Outcome, Error> {
        super::read_line::<Dummy, Dummy, Dummy, W>(
            self.buffer,
            None,
            None,
            None,
            &mut self.writer.unwrap(),
        )
    }
}

impl<T, B> Builder for WithOverrider<T, B>
where
    T: Overrider,
    B: Builder,
{
    impl_builder!(base);
    impl_builder!(extensions);

    fn read_line(self) -> Result<Outcome, Error> {
        self.base
            .chain_read_line::<T, Dummy, Dummy>(Some(&self.overrider), None, None)
    }
}

impl<T, B> Builder for WithCompleter<T, B>
where
    T: Completer,
    B: Builder,
{
    impl_builder!(base);
    impl_builder!(extensions);

    fn read_line(self) -> Result<Outcome, Error> {
        self.base
            .chain_read_line::<Dummy, T, Dummy>(None, Some(&self.completer), None)
    }
}

impl<T, B> Builder for WithSuggester<T, B>
where
    T: Suggester,
    B: Builder,
{
    impl_builder!(base);
    impl_builder!(extensions);

    fn read_line(self) -> Result<Outcome, Error> {
        self.base
            .chain_read_line::<Dummy, Dummy, T>(None, None, Some(&self.suggester))
    }
}

impl<T, B> Builder for WithRefOverrider<'_, T, B>
where
    T: Overrider + ?Sized,
    B: Builder,
{
    impl_builder!(base);
    impl_builder!(extensions);

    fn read_line(self) -> Result<Outcome, Error> {
        self.base
            .chain_read_line::<T, Dummy, Dummy>(Some(self.overrider), None, None)
    }
}

impl<T, B> Builder for WithRefCompleter<'_, T, B>
where
    T: Completer + ?Sized,
    B: Builder,
{
    impl_builder!(base);
    impl_builder!(extensions);

    fn read_line(self) -> Result<Outcome, Error> {
        self.base
            .chain_read_line::<Dummy, T, Dummy>(None, Some(self.completer), None)
    }
}

impl<T, B> Builder for WithRefSuggester<'_, T, B>
where
    T: Suggester + ?Sized,
    B: Builder,
{
    impl_builder!(base);
    impl_builder!(extensions);

    fn read_line(self) -> Result<Outcome, Error> {
        self.base
            .chain_read_line::<Dummy, Dummy, T>(None, None, Some(self.suggester))
    }
}

impl<W: Writer> ChainedLineReader for Prompt<W> {
    default fn chain_read_line<O, C, S>(
        self,
        overrider: Option<&O>,
        completer: Option<&C>,
        suggester: Option<&S>,
    ) -> Result<Outcome, Error>
    where
        O: Overrider + ?Sized,
        C: Completer + ?Sized,
        S: Suggester + ?Sized,
    {
        super::read_line(
            self.buffer,
            overrider,
            completer,
            suggester,
            &mut self.writer.unwrap(),
        )
    }
}

impl ChainedLineReader for Prompt<RuclineWriter<'_>> {
    default fn chain_read_line<O, C, S>(
        self,
        overrider: Option<&O>,
        completer: Option<&C>,
        suggester: Option<&S>,
    ) -> Result<Outcome, Error>
    where
        O: Overrider + ?Sized,
        C: Completer + ?Sized,
        S: Suggester + ?Sized,
    {
        let mut writer = RuclineWriter::new(self.erase_after_read, self.prompt.as_deref());
        super::read_line(
            self.buffer,
            overrider,
            completer,
            suggester,
            &mut writer,
        )
    }
}

impl<T, B> ChainedLineReader for WithOverrider<T, B>
where
    T: Overrider,
    B: Builder,
{
    fn chain_read_line<O, C, S>(
        self,
        overrider: Option<&O>,
        completer: Option<&C>,
        suggester: Option<&S>,
    ) -> Result<Outcome, Error>
    where
        O: Overrider + ?Sized,
        C: Completer + ?Sized,
        S: Suggester + ?Sized,
    {
        if overrider.is_some() {
            self.base.chain_read_line(overrider, completer, suggester)
        } else {
            self.base
                .chain_read_line(Some(&self.overrider), completer, suggester)
        }
    }
}

impl<T, B> ChainedLineReader for WithCompleter<T, B>
where
    T: Completer,
    B: Builder,
{
    fn chain_read_line<O, C, S>(
        self,
        overrider: Option<&O>,
        completer: Option<&C>,
        suggester: Option<&S>,
    ) -> Result<Outcome, Error>
    where
        O: Overrider + ?Sized,
        C: Completer + ?Sized,
        S: Suggester + ?Sized,
    {
        if completer.is_some() {
            self.base.chain_read_line(overrider, completer, suggester)
        } else {
            self.base
                .chain_read_line(overrider, Some(&self.completer), suggester)
        }
    }
}

impl<T, B> ChainedLineReader for WithSuggester<T, B>
where
    T: Suggester,
    B: Builder,
{
    fn chain_read_line<O, C, S>(
        self,
        overrider: Option<&O>,
        completer: Option<&C>,
        suggester: Option<&S>,
    ) -> Result<Outcome, Error>
    where
        O: Overrider + ?Sized,
        C: Completer + ?Sized,
        S: Suggester + ?Sized,
    {
        if suggester.is_some() {
            self.base.chain_read_line(overrider, completer, suggester)
        } else {
            self.base
                .chain_read_line(overrider, completer, Some(&self.suggester))
        }
    }
}

impl<T, B> ChainedLineReader for WithRefOverrider<'_, T, B>
where
    T: Overrider + ?Sized,
    B: Builder,
{
    fn chain_read_line<O, C, S>(
        self,
        overrider: Option<&O>,
        completer: Option<&C>,
        suggester: Option<&S>,
    ) -> Result<Outcome, Error>
    where
        O: Overrider + ?Sized,
        C: Completer + ?Sized,
        S: Suggester + ?Sized,
    {
        if overrider.is_some() {
            self.base.chain_read_line(overrider, completer, suggester)
        } else {
            self.base
                .chain_read_line(Some(self.overrider), completer, suggester)
        }
    }
}

impl<T, B> ChainedLineReader for WithRefCompleter<'_, T, B>
where
    T: Completer + ?Sized,
    B: Builder,
{
    fn chain_read_line<O, C, S>(
        self,
        overrider: Option<&O>,
        completer: Option<&C>,
        suggester: Option<&S>,
    ) -> Result<Outcome, Error>
    where
        O: Overrider + ?Sized,
        C: Completer + ?Sized,
        S: Suggester + ?Sized,
    {
        if completer.is_some() {
            self.base.chain_read_line(overrider, completer, suggester)
        } else {
            self.base
                .chain_read_line(overrider, Some(self.completer), suggester)
        }
    }
}

impl<T, B> ChainedLineReader for WithRefSuggester<'_, T, B>
where
    T: Suggester + ?Sized,
    B: Builder,
{
    fn chain_read_line<O, C, S>(
        self,
        overrider: Option<&O>,
        completer: Option<&C>,
        suggester: Option<&S>,
    ) -> Result<Outcome, Error>
    where
        O: Overrider + ?Sized,
        C: Completer + ?Sized,
        S: Suggester + ?Sized,
    {
        if suggester.is_some() {
            self.base.chain_read_line(overrider, completer, suggester)
        } else {
            self.base
                .chain_read_line(overrider, completer, Some(self.suggester))
        }
    }
}

pub struct Closure<'a, F, R>
where
    F: Fn(&Buffer) -> R,
{
    closure: F,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a, F, R> Completer for Closure<'a, F, Option<R>>
where
    F: Fn(&Buffer) -> Option<R>,
    R: Into<std::borrow::Cow<'a, str>>,
{
    fn complete_for(&self, buffer: &Buffer) -> Option<std::borrow::Cow<'_, str>> {
        (self.closure)(buffer).map(Into::into)
    }
}

impl<'a, F, R> Suggester for Closure<'a, F, Vec<R>>
where
    F: Fn(&Buffer) -> Vec<R>,
    R: Into<std::borrow::Cow<'a, str>>,
{
    fn suggest_for(&self, buffer: &Buffer) -> Vec<std::borrow::Cow<'_, str>> {
        (self.closure)(buffer).into_iter().map(Into::into).collect()
    }
}

struct Dummy;

impl Overrider for Dummy {
    fn override_for(&self, _: crate::actions::Event, _: &Buffer) -> Option<crate::actions::Action> {
        unimplemented!()
    }
}

impl Completer for Dummy {
    fn complete_for(&self, _: &Buffer) -> Option<std::borrow::Cow<'_, str>> {
        unimplemented!()
    }
}

impl Suggester for Dummy {
    fn suggest_for(&self, _: &Buffer) -> Vec<std::borrow::Cow<'_, str>> {
        unimplemented!()
    }
}

#[cfg(test)]
mod test {
    use super::{Builder, ChainedLineReader, Prompt};

    #[test]
    fn accept_decorated_prompt() {
        use crossterm::style::Colorize;

        let mut prompt = Prompt::from("My prompt".green());

        assert_eq!(
            prompt.prompt.take().unwrap().len(),
            format!("{}", "My prompt".green()).len()
        );
    }

    #[test]
    fn last_hook_is_used() {
        use super::{
            Action, Buffer, Closure, Completer, Dummy, Error, Event, Outcome, Overrider, Suggester,
            WithCompleter, WithOverrider, WithRefCompleter, WithRefOverrider, WithRefSuggester,
            WithSuggester,
        };
        use crossterm::event::KeyCode::Tab;

        struct MockBuilder;
        impl Builder for MockBuilder {
            fn buffer(self, _: Buffer) -> Self {
                unimplemented!()
            }

            fn erase_after_read(self, _: bool) -> Self {
                unimplemented!()
            }

            fn read_line(self) -> Result<Outcome, Error> {
                unimplemented!()
            }

            impl_builder!(extensions);
        }

        impl ChainedLineReader for MockBuilder {
            fn chain_read_line<O, C, S>(
                self,
                overrider: Option<&O>,
                completer: Option<&C>,
                suggester: Option<&S>,
            ) -> Result<Outcome, Error>
            where
                O: Overrider + ?Sized,
                C: Completer + ?Sized,
                S: Suggester + ?Sized,
            {
                assert_eq!(
                    overrider
                        .unwrap()
                        .override_for(Event::from(Tab), &Buffer::new())
                        .unwrap(),
                    Action::Accept
                );
                assert_eq!(
                    completer.unwrap().complete_for(&Buffer::from("-")).unwrap(),
                    "expected"
                );
                assert_eq!(
                    suggester.unwrap().suggest_for(&Buffer::from("-"))[0],
                    "-expected"
                );
                Ok(Outcome::Accepted(String::new()))
            }
        }

        struct MockOverrider;

        impl Overrider for MockOverrider {
            fn override_for(&self, _: Event, _: &Buffer) -> Option<Action> {
                Some(Action::Cancel)
            }
        }

        MockBuilder {}
            .overrider_ref(&MockOverrider)
            .overrider(MockOverrider)
            .overrider_fn(|_, _| Some(Action::Accept))
            .completer_fn(|_| Some("-unexpected"))
            .completer(vec!["-unexpected"])
            .completer_ref(&vec!["-expected"])
            .suggester(vec!["-unexpected"])
            .suggester_fn(|_| vec!["-unexpected"])
            .suggester_ref(&vec!["-expected"])
            .chain_read_line::<Dummy, Dummy, Dummy>(None, None, None)
            .unwrap();
    }
}
