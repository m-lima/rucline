use super::Outcome;

use crate::actions::Overrider;
use crate::buffer::Buffer;
use crate::completion::{Completer, Suggester};

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
        fn overrider<'o, O: Overrider>(self, overrider: &'o O) -> WithOverrider<'o, O, Self::Base> {
            WithOverrider {
                base: self,
                overrider,
            }
        }

        fn completer<'c, C: Completer>(self, completer: &'c C) -> WithCompleter<'c, C, Self::Base> {
            WithCompleter {
                base: self,
                completer,
            }
        }

        fn suggester<'s, S: Suggester>(self, suggester: &'s S) -> WithSuggester<'s, S, Self::Base> {
            WithSuggester {
                base: self,
                suggester,
            }
        }
    };
}

pub trait Builder: ChainedLineReader {
    type Base: Builder;

    fn new() -> Base {
        Base {
            prompt: None,
            buffer: None,
            erase_after_read: false,
        }
    }

    fn from<S: ToString>(s: S) -> Base {
        Base {
            prompt: Some(s.to_string()),
            buffer: None,
            erase_after_read: false,
        }
    }

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
    fn overrider<'o, O: Overrider>(self, overrider: &'o O) -> WithOverrider<'o, O, Self::Base>;

    /// Sets the in-line completion provider.
    ///
    /// # Arguments
    ///
    /// * [`completer`] - The new completer
    ///
    /// [`Completer`]: ../completion/trait.Completer.html
    fn completer<'c, C: Completer>(self, completer: &'c C) -> WithCompleter<'c, C, Self::Base>;

    /// Sets the drop-down suggestion provider.
    ///
    /// # Arguments
    ///
    /// * [`suggester`] - The new suggester
    ///
    /// [`Suggester`]: ../completion/trait.Suggester.html
    fn suggester<'s, S: Suggester>(self, suggester: &'s S) -> WithSuggester<'s, S, Self::Base>;

    fn read_line(self) -> Result<Outcome, crate::ErrorKind>;
}

pub trait ChainedLineReader {
    fn chain_read_line<'o, 'c, 's, O, C, S>(
        self,
        overrider: Option<&'o O>,
        completer: Option<&'c C>,
        suggester: Option<&'s S>,
    ) -> Result<Outcome, crate::ErrorKind>
    where
        O: Overrider + ?Sized,
        C: Completer + ?Sized,
        S: Suggester + ?Sized;
}

pub struct Base {
    prompt: Option<String>,
    buffer: Option<Buffer>,
    erase_after_read: bool,
}

pub struct WithOverrider<'o, O, B>
where
    O: Overrider + ?Sized,
    B: Builder,
{
    base: B,
    overrider: &'o O,
}

pub struct WithCompleter<'c, C, B>
where
    C: Completer + ?Sized,
    B: Builder,
{
    base: B,
    completer: &'c C,
}

pub struct WithSuggester<'s, S, B>
where
    S: Suggester + ?Sized,
    B: Builder,
{
    base: B,
    suggester: &'s S,
}

impl Builder for Base {
    type Base = Self;

    fn buffer(mut self, buffer: Buffer) -> Self {
        self.buffer = Some(buffer);
        self
    }

    fn erase_after_read(mut self, erase_after_read: bool) -> Self {
        self.erase_after_read = erase_after_read;
        self
    }

    boiler_plate!(extensions);

    fn read_line(self) -> Result<Outcome, crate::ErrorKind> {
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

impl<T, B> Builder for WithOverrider<'_, T, B>
where
    T: Overrider + ?Sized,
    B: Builder,
{
    type Base = Self;

    boiler_plate!(base);
    boiler_plate!(extensions);

    fn read_line(self) -> Result<Outcome, crate::ErrorKind> {
        self.base
            .chain_read_line::<T, Dummy, Dummy>(Some(self.overrider), None, None)
    }
}

impl<T, B> Builder for WithCompleter<'_, T, B>
where
    T: Completer + ?Sized,
    B: Builder,
{
    type Base = Self;

    boiler_plate!(base);
    boiler_plate!(extensions);

    fn read_line(self) -> Result<Outcome, crate::ErrorKind> {
        self.base
            .chain_read_line::<Dummy, T, Dummy>(None, Some(self.completer), None)
    }
}

impl<T, B> Builder for WithSuggester<'_, T, B>
where
    T: Suggester + ?Sized,
    B: Builder,
{
    type Base = Self;

    boiler_plate!(base);
    boiler_plate!(extensions);

    fn read_line(self) -> Result<Outcome, crate::ErrorKind> {
        self.base
            .chain_read_line::<Dummy, Dummy, T>(None, None, Some(self.suggester))
    }
}

impl ChainedLineReader for Base {
    fn chain_read_line<'o, 'c, 's, O, C, S>(
        self,
        overrider: Option<&'o O>,
        completer: Option<&'c C>,
        suggester: Option<&'s S>,
    ) -> Result<Outcome, crate::ErrorKind>
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

impl<O, B> ChainedLineReader for WithOverrider<'_, O, B>
where
    O: Overrider + ?Sized,
    B: Builder,
{
    fn chain_read_line<'r, 'c, 's, R, C, S>(
        self,
        _: Option<&'r R>,
        completer: Option<&'c C>,
        suggester: Option<&'s S>,
    ) -> Result<Outcome, crate::ErrorKind>
    where
        R: Overrider + ?Sized,
        C: Completer + ?Sized,
        S: Suggester + ?Sized,
    {
        self.base
            .chain_read_line(Some(self.overrider), completer, suggester)
    }
}

impl<C, B> ChainedLineReader for WithCompleter<'_, C, B>
where
    C: Completer + ?Sized,
    B: Builder,
{
    fn chain_read_line<'o, 'r, 's, O, R, S>(
        self,
        overrider: Option<&'o O>,
        _: Option<&'r R>,
        suggester: Option<&'s S>,
    ) -> Result<Outcome, crate::ErrorKind>
    where
        O: Overrider + ?Sized,
        R: Completer + ?Sized,
        S: Suggester + ?Sized,
    {
        self.base
            .chain_read_line(overrider, Some(self.completer), suggester)
    }
}

impl<S, B> ChainedLineReader for WithSuggester<'_, S, B>
where
    S: Suggester + ?Sized,
    B: Builder,
{
    fn chain_read_line<'o, 'c, 'r, O, C, R>(
        self,
        overrider: Option<&'o O>,
        completer: Option<&'c C>,
        _: Option<&'r R>,
    ) -> Result<Outcome, crate::ErrorKind>
    where
        O: Overrider + ?Sized,
        C: Completer + ?Sized,
        R: Suggester + ?Sized,
    {
        self.base
            .chain_read_line(overrider, completer, Some(self.suggester))
    }
}

struct Dummy;

impl Overrider for Dummy {
    fn override_for(
        &self,
        _: crate::actions::Event,
        _: &dyn crate::Context,
    ) -> Option<crate::actions::Action> {
        unimplemented!()
    }
}

impl Completer for Dummy {
    fn complete_for(&self, _: &dyn crate::Context) -> Option<&str> {
        unimplemented!()
    }
}

impl Suggester for Dummy {
    fn suggest_for(&self, _: &dyn crate::Context) -> Vec<&str> {
        unimplemented!()
    }
}
