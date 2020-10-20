use super::Outcome;

use crate::actions::{Action, Event, Overrider};
use crate::buffer::Buffer;
use crate::completion::{Completer, Suggester};
use crate::ErrorKind;

macro_rules! boiler_plate {
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
            O: Fn(Event, &Buffer) -> Option<Action>,
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

pub trait Builder: ChainedLineReader + Sized {
    /// Prepopulates the prompt input with `buffer`.
    ///
    /// # Arguments
    ///
    /// * [`buffer`] - A buffer to be used when displaying the prompt
    ///
    /// [`Buffer`]: ../buffer/trait.Buffer.html
    fn buffer(self, buffer: Buffer) -> Self;

    /// Controls if the prompt shall be erased after user input.
    ///
    /// If set to `false` (default), after user input, the terminal will receive a new line
    /// after the prompt text and the user input. Any drop-down completions will be removed,
    /// however.
    ///
    /// If set to `true`, the whole prompt and input will be erased. The cursor returns to the
    /// original position as if nothing happened.
    fn erase_after_read(self, erase_after_read: bool) -> Self;

    /// Modifies the behavior of the prompt by setting a [`Overrider`].
    ///
    /// # Arguments
    ///
    /// * [`overrider`] - The new overrider
    ///
    /// [`Overrider`]: ../actions/trait.Overrider.html
    fn overrider<O: Overrider>(self, overrider: O) -> WithOverrider<O, Self>;

    /// Modifies the behavior of the prompt by setting a [`Overrider`] closure.
    ///
    /// # Arguments
    ///
    /// * [`overrider`] - The new overrider
    ///
    /// [`Overrider`]: ../actions/trait.Overrider.html
    fn overrider_fn<O>(self, overrider: O) -> WithOverrider<O, Self>
    where
        O: Fn(Event, &Buffer) -> Option<Action>;

    /// Modifies the behavior of the prompt by setting a [`Overrider`] reference.
    ///
    /// # Arguments
    ///
    /// * [`overrider`] - The new overrider
    ///
    /// [`Overrider`]: ../actions/trait.Overrider.html
    fn overrider_ref<O: Overrider>(self, overrider: &O) -> WithRefOverrider<'_, O, Self>;

    /// Sets the in-line completion provider.
    ///
    /// # Arguments
    ///
    /// * [`completer`] - The new completer
    ///
    /// [`Completer`]: ../completion/trait.Completer.html
    fn completer<C: Completer>(self, completer: C) -> WithCompleter<C, Self>;

    /// Sets the in-line completion provider.
    ///
    /// # Arguments
    ///
    /// * [`completer`] - The new completer
    ///
    /// [`Completer`]: ../completion/trait.Completer.html
    fn completer_fn<'a, F, R>(self, closure: F) -> WithCompleter<Closure<'a, F, Option<R>>, Self>
    where
        F: Fn(&Buffer) -> Option<R>,
        R: Into<std::borrow::Cow<'a, str>>;

    /// Sets the in-line completion provider.
    ///
    /// # Arguments
    ///
    /// * [`completer`] - The new completer
    ///
    /// [`Completer`]: ../completion/trait.Completer.html
    fn completer_ref<C: Completer>(self, completer: &C) -> WithRefCompleter<'_, C, Self>;

    /// Sets the drop-down suggestion provider.
    ///
    /// # Arguments
    ///
    /// * [`suggester`] - The new suggester
    ///
    /// [`Suggester`]: ../completion/trait.Suggester.html
    fn suggester<S: Suggester>(self, suggester: S) -> WithSuggester<S, Self>;

    /// Sets the drop-down suggestion provider.
    ///
    /// # Arguments
    ///
    /// * [`suggester`] - The new suggester
    ///
    /// [`Suggester`]: ../completion/trait.Suggester.html
    fn suggester_fn<'a, F, R>(self, closure: F) -> WithSuggester<Closure<'a, F, Vec<R>>, Self>
    where
        F: Fn(&Buffer) -> Vec<R>,
        R: Into<std::borrow::Cow<'a, str>>;

    /// Sets the drop-down suggestion provider.
    ///
    /// # Arguments
    ///
    /// * [`suggester`] - The new suggester
    ///
    /// [`Suggester`]: ../completion/trait.Suggester.html
    fn suggester_ref<S: Suggester>(self, suggester: &S) -> WithRefSuggester<'_, S, Self>;

    fn read_line(self) -> Result<Outcome, ErrorKind>;
}

pub trait ChainedLineReader {
    fn chain_read_line<O, C, S>(
        self,
        overrider: Option<&O>,
        completer: Option<&C>,
        suggester: Option<&S>,
    ) -> Result<Outcome, ErrorKind>
    where
        O: Overrider + ?Sized,
        C: Completer + ?Sized,
        S: Suggester + ?Sized;
}

pub struct Prompt {
    prompt: Option<String>,
    buffer: Option<Buffer>,
    erase_after_read: bool,
}

impl Prompt {
    pub fn new() -> Self {
        Self {
            prompt: None,
            buffer: None,
            erase_after_read: false,
        }
    }

    pub fn from<S: ToString>(s: S) -> Self {
        Self {
            prompt: Some(s.to_string()),
            buffer: None,
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

impl Builder for Prompt {
    fn buffer(mut self, buffer: Buffer) -> Self {
        self.buffer = Some(buffer);
        self
    }

    fn erase_after_read(mut self, erase_after_read: bool) -> Self {
        self.erase_after_read = erase_after_read;
        self
    }

    boiler_plate!(extensions);

    fn read_line(self) -> Result<Outcome, ErrorKind> {
        super::read_line::<Dummy, Dummy, Dummy>(
            self.prompt.as_deref(),
            self.buffer,
            self.erase_after_read,
            None,
            None,
            None,
        )
    }
}

impl<T, B> Builder for WithOverrider<T, B>
where
    T: Overrider,
    B: Builder,
{
    boiler_plate!(base);
    boiler_plate!(extensions);

    fn read_line(self) -> Result<Outcome, ErrorKind> {
        self.base
            .chain_read_line::<T, Dummy, Dummy>(Some(&self.overrider), None, None)
    }
}

impl<T, B> Builder for WithCompleter<T, B>
where
    T: Completer,
    B: Builder,
{
    boiler_plate!(base);
    boiler_plate!(extensions);

    fn read_line(self) -> Result<Outcome, ErrorKind> {
        self.base
            .chain_read_line::<Dummy, T, Dummy>(None, Some(&self.completer), None)
    }
}

impl<T, B> Builder for WithSuggester<T, B>
where
    T: Suggester,
    B: Builder,
{
    boiler_plate!(base);
    boiler_plate!(extensions);

    fn read_line(self) -> Result<Outcome, ErrorKind> {
        self.base
            .chain_read_line::<Dummy, Dummy, T>(None, None, Some(&self.suggester))
    }
}

impl<T, B> Builder for WithRefOverrider<'_, T, B>
where
    T: Overrider + ?Sized,
    B: Builder,
{
    boiler_plate!(base);
    boiler_plate!(extensions);

    fn read_line(self) -> Result<Outcome, ErrorKind> {
        self.base
            .chain_read_line::<T, Dummy, Dummy>(Some(self.overrider), None, None)
    }
}

impl<T, B> Builder for WithRefCompleter<'_, T, B>
where
    T: Completer + ?Sized,
    B: Builder,
{
    boiler_plate!(base);
    boiler_plate!(extensions);

    fn read_line(self) -> Result<Outcome, ErrorKind> {
        self.base
            .chain_read_line::<Dummy, T, Dummy>(None, Some(self.completer), None)
    }
}

impl<T, B> Builder for WithRefSuggester<'_, T, B>
where
    T: Suggester + ?Sized,
    B: Builder,
{
    boiler_plate!(base);
    boiler_plate!(extensions);

    fn read_line(self) -> Result<Outcome, ErrorKind> {
        self.base
            .chain_read_line::<Dummy, Dummy, T>(None, None, Some(self.suggester))
    }
}

impl ChainedLineReader for Prompt {
    fn chain_read_line<O, C, S>(
        self,
        overrider: Option<&O>,
        completer: Option<&C>,
        suggester: Option<&S>,
    ) -> Result<Outcome, ErrorKind>
    where
        O: Overrider + ?Sized,
        C: Completer + ?Sized,
        S: Suggester + ?Sized,
    {
        super::read_line(
            self.prompt.as_deref(),
            self.buffer,
            self.erase_after_read,
            overrider,
            completer,
            suggester,
        )
    }
}

impl<O, B> ChainedLineReader for WithOverrider<O, B>
where
    O: Overrider,
    B: Builder,
{
    fn chain_read_line<R, C, S>(
        self,
        _: Option<&R>,
        completer: Option<&C>,
        suggester: Option<&S>,
    ) -> Result<Outcome, ErrorKind>
    where
        R: Overrider + ?Sized,
        C: Completer + ?Sized,
        S: Suggester + ?Sized,
    {
        self.base
            .chain_read_line(Some(&self.overrider), completer, suggester)
    }
}

impl<C, B> ChainedLineReader for WithCompleter<C, B>
where
    C: Completer,
    B: Builder,
{
    fn chain_read_line<O, R, S>(
        self,
        overrider: Option<&O>,
        _: Option<&R>,
        suggester: Option<&S>,
    ) -> Result<Outcome, ErrorKind>
    where
        O: Overrider + ?Sized,
        R: Completer + ?Sized,
        S: Suggester + ?Sized,
    {
        self.base
            .chain_read_line(overrider, Some(&self.completer), suggester)
    }
}

impl<S, B> ChainedLineReader for WithSuggester<S, B>
where
    S: Suggester,
    B: Builder,
{
    fn chain_read_line<O, C, R>(
        self,
        overrider: Option<&O>,
        completer: Option<&C>,
        _: Option<&R>,
    ) -> Result<Outcome, ErrorKind>
    where
        O: Overrider + ?Sized,
        C: Completer + ?Sized,
        R: Suggester + ?Sized,
    {
        self.base
            .chain_read_line(overrider, completer, Some(&self.suggester))
    }
}

impl<O, B> ChainedLineReader for WithRefOverrider<'_, O, B>
where
    O: Overrider + ?Sized,
    B: Builder,
{
    fn chain_read_line<R, C, S>(
        self,
        _: Option<&R>,
        completer: Option<&C>,
        suggester: Option<&S>,
    ) -> Result<Outcome, ErrorKind>
    where
        R: Overrider + ?Sized,
        C: Completer + ?Sized,
        S: Suggester + ?Sized,
    {
        self.base
            .chain_read_line(Some(self.overrider), completer, suggester)
    }
}

impl<C, B> ChainedLineReader for WithRefCompleter<'_, C, B>
where
    C: Completer + ?Sized,
    B: Builder,
{
    fn chain_read_line<O, R, S>(
        self,
        overrider: Option<&O>,
        _: Option<&R>,
        suggester: Option<&S>,
    ) -> Result<Outcome, ErrorKind>
    where
        O: Overrider + ?Sized,
        R: Completer + ?Sized,
        S: Suggester + ?Sized,
    {
        self.base
            .chain_read_line(overrider, Some(self.completer), suggester)
    }
}

impl<S, B> ChainedLineReader for WithRefSuggester<'_, S, B>
where
    S: Suggester + ?Sized,
    B: Builder,
{
    fn chain_read_line<O, C, R>(
        self,
        overrider: Option<&O>,
        completer: Option<&C>,
        _: Option<&R>,
    ) -> Result<Outcome, ErrorKind>
    where
        O: Overrider + ?Sized,
        C: Completer + ?Sized,
        R: Suggester + ?Sized,
    {
        self.base
            .chain_read_line(overrider, completer, Some(self.suggester))
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
    fn override_for(
        &self,
        _: crate::actions::Event,
        _: &crate::Buffer,
    ) -> Option<crate::actions::Action> {
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
