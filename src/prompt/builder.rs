use super::Outcome;

use crate::actions::Overrider;
use crate::buffer::Buffer;
use crate::completion::{Completer, Suggester};

pub struct Dummy;

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

/// Represents and stores a prompt that shall be presented to the user for input.
///
/// When built, the prompt will have no customization or completions. Also the default
/// [`erase_after_read`] is `true`.
///
/// [`erase_after_read`]: struct.Prompt.html#method.erase_after_read
pub struct Builder<'o, 'c, 's, O, C, S>
where
    O: Overrider + ?Sized,
    C: Completer + ?Sized,
    S: Suggester + ?Sized,
{
    prompt: Option<String>,
    buffer: Option<Buffer>,
    overrider: Option<&'o O>,
    completer: Option<&'c C>,
    suggester: Option<&'s S>,
    erase_after_read: bool,
}

impl<'o, 'c, 's, O, C, S> Builder<'o, 'c, 's, O, C, S>
where
    O: Overrider + ?Sized,
    C: Completer + ?Sized,
    S: Suggester + ?Sized,
{
    pub fn new() -> Self {
        Self {
            prompt: None,
            buffer: None,
            overrider: None,
            completer: None,
            suggester: None,
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
    #[must_use]
    pub fn buffer(mut self, buffer: Buffer) -> Self {
        self.buffer = Some(buffer);
        self
    }

    /// Modifies the behavior of the prompt by setting a [`Overrider`].
    ///
    /// # Arguments
    ///
    /// * [`overrider`] - The new overrider
    ///
    /// [`Overrider`]: ../actions/trait.Overrider.html
    #[must_use]
    pub fn overrider<'no, NO: Overrider>(
        self,
        overrider: &'no NO,
    ) -> Builder<'no, 'c, 's, NO, C, S> {
        Builder {
            prompt: self.prompt,
            buffer: self.buffer,
            overrider: Some(overrider),
            completer: self.completer,
            suggester: self.suggester,
            erase_after_read: self.erase_after_read,
        }
    }

    /// Sets the in-line completion provider.
    ///
    /// # Arguments
    ///
    /// * [`completer`] - The new completer
    ///
    /// [`Completer`]: ../completion/trait.Completer.html
    #[must_use]
    pub fn completer<'nc, NC: Completer>(
        self,
        completer: &'nc NC,
    ) -> Builder<'o, 'nc, 's, O, NC, S> {
        Builder {
            prompt: self.prompt,
            buffer: self.buffer,
            overrider: self.overrider,
            completer: Some(completer),
            suggester: self.suggester,
            erase_after_read: self.erase_after_read,
        }
    }

    /// Sets the drop-down suggestion provider.
    ///
    /// # Arguments
    ///
    /// * [`suggester`] - The new suggester
    ///
    /// [`Suggester`]: ../completion/trait.Suggester.html
    #[must_use]
    pub fn suggester<'ns, NS: Suggester>(
        self,
        suggester: &'ns NS,
    ) -> Builder<'o, 'c, 'ns, O, C, NS> {
        Builder {
            prompt: self.prompt,
            buffer: self.buffer,
            overrider: self.overrider,
            completer: self.completer,
            suggester: Some(suggester),
            erase_after_read: self.erase_after_read,
        }
    }

    /// Controls if the prompt shall be erased after user input.
    ///
    /// If set to `false` (default), after user input, the terminal will receive a new line
    /// after the prompt text and the user input. Any drop-down completions will be removed,
    /// however.
    ///
    /// If set to `true`, the whole prompt and input will be erased. The cursor returns to the
    /// original position as if nothing happened.
    pub fn erase_after_read(mut self, erase_after_read: bool) -> Self {
        self.erase_after_read = erase_after_read;
        self
    }
}

impl<O, C, S> super::LineReader for Builder<'_, '_, '_, O, C, S>
where
    O: Overrider + ?Sized,
    C: Completer + ?Sized,
    S: Suggester + ?Sized,
{
    fn read_line(self) -> Result<Outcome, crate::ErrorKind> {
        super::read_line(
            self.prompt.as_deref(),
            self.buffer,
            self.erase_after_read,
            self.overrider,
            self.completer,
            self.suggester,
        )
    }
}

impl<S: ToString> std::convert::From<S> for Builder<'_, '_, '_, Dummy, Dummy, Dummy> {
    fn from(string: S) -> Self {
        Self {
            prompt: Some(string.to_string()),
            buffer: None,
            overrider: None,
            completer: None,
            suggester: None,
            erase_after_read: false,
        }
    }
}

//use super::{LineReader, Outcome};

//use crate::actions::Overrider;
//use crate::buffer::Buffer;
//use crate::completion::{Completer, Suggester};

//pub trait Builder: LineReader {
//    fn buffer(self, buffer: Buffer) -> Box<dyn Builder>;
//    fn overrider<'o>(self, overrider: &'o dyn Overrider) -> Box<dyn Builder>;
//}

//trait ChainedLineReader {
//    fn chain_read_line<'o, 'c, 's, O, C, S>(
//        self,
//        overrider: Option<&'o O>,
//        completer: Option<&'c C>,
//        suggester: Option<&'s S>,
//    ) -> Result<Outcome, crate::ErrorKind>
//    where
//        O: Overrider + ?Sized,
//        C: Completer + ?Sized,
//        S: Suggester + ?Sized;
//}

//struct Base {
//    prompt: Option<String>,
//    buffer: Option<Buffer>,
//    erase_after_read: bool,
//}

//struct WithOverrider<'o, O, B>
//where
//    O: Overrider + ?Sized,
//    B: Builder + ChainedLineReader,
//{
//    base: B,
//    overrider: &'o O,
//}

//struct WithCompleter<'c, C, B>
//where
//    C: Completer + ?Sized,
//    B: Builder + ChainedLineReader,
//{
//    base: B,
//    completer: &'c C,
//}

//struct WithSuggester<'s, S, B>
//where
//    S: Suggester + ?Sized,
//    B: Builder + ChainedLineReader,
//{
//    base: B,
//    suggester: &'s S,
//}

//impl LineReader for Base {
//    fn read_line(self) -> Result<Outcome, crate::ErrorKind> {
//        super::read_line::<__Dummy, __Dummy, __Dummy>(
//            self.prompt.as_deref(),
//            self.buffer,
//            self.erase_after_read,
//            None,
//            None,
//            None,
//        )
//    }
//}

//impl Builder for Base {
//    fn buffer(mut self, buffer: Buffer) -> Box<dyn Builder> {
//        self.buffer = Some(buffer);
//        Box::new(self)
//    }

//    fn overrider<'o>(self, overrider: &'o dyn Overrider) -> Box<dyn Builder> {
//        Box::new(WithOverrider::<'o> {
//            base: self,
//            overrider: overrider,
//        })
//    }
//}

//impl ChainedLineReader for Base {
//    fn chain_read_line<'o, 'c, 's, O, C, S>(
//        self,
//        overrider: Option<&'o O>,
//        completer: Option<&'c C>,
//        suggester: Option<&'s S>,
//    ) -> Result<Outcome, crate::ErrorKind>
//    where
//        O: Overrider + ?Sized,
//        C: Completer + ?Sized,
//        S: Suggester + ?Sized,
//    {
//        super::read_line(
//            self.prompt.as_deref(),
//            self.buffer,
//            self.erase_after_read,
//            overrider,
//            completer,
//            suggester,
//        )
//    }
//}

//impl<O, B> LineReader for WithOverrider<'_, O, B>
//where
//    O: Overrider + ?Sized,
//    B: Builder + ChainedLineReader,
//{
//    fn read_line(self) -> Result<Outcome, crate::ErrorKind> {
//        self.base
//            .chain_read_line::<O, __Dummy, __Dummy>(Some(self.overrider), None, None)
//    }
//}

//impl<O, B> Builder for WithOverrider<'_, O, B>
//where
//    O: Overrider + ?Sized,
//    B: Builder + ChainedLineReader,
//{
//    fn buffer(mut self, buffer: Buffer) -> Box<dyn Builder> {
//        Box::new(WithOverrider {
//            base: self.base.buffer(buffer),
//            overrider: self.overrider,
//        })
//    }

//    fn overrider<'o>(self, overrider: &'o dyn Overrider) -> Box<dyn Builder> {
//        Box::new(WithOverrider {
//            base: self,
//            overrider,
//        })
//    }
//}

//impl<O, B> ChainedLineReader for WithOverrider<'_, O, B>
//where
//    O: Overrider + ?Sized,
//    B: Builder + ChainedLineReader,
//{
//    fn chain_read_line<'r, 'c, 's, R, C, S>(
//        self,
//        _: Option<&'r R>,
//        completer: Option<&'c C>,
//        suggester: Option<&'s S>,
//    ) -> Result<Outcome, crate::ErrorKind>
//    where
//        R: Overrider + ?Sized,
//        C: Completer + ?Sized,
//        S: Suggester + ?Sized,
//    {
//        self.base
//            .chain_read_line(Some(self.overrider), completer, suggester)
//    }
//}

//impl<C, B> LineReader for WithCompleter<'_, C, B>
//where
//    C: Completer + ?Sized,
//    B: Builder + ChainedLineReader,
//{
//    fn read_line(self) -> Result<Outcome, crate::ErrorKind> {
//        self.base
//            .chain_read_line::<__Dummy, C, __Dummy>(None, Some(self.completer), None)
//    }
//}

//impl<C, B> ChainedLineReader for WithCompleter<'_, C, B>
//where
//    C: Completer + ?Sized,
//    B: Builder + ChainedLineReader,
//{
//    fn chain_read_line<'o, 'r, 's, O, R, S>(
//        self,
//        overrider: Option<&'o O>,
//        _: Option<&'r R>,
//        suggester: Option<&'s S>,
//    ) -> Result<Outcome, crate::ErrorKind>
//    where
//        O: Overrider + ?Sized,
//        R: Completer + ?Sized,
//        S: Suggester + ?Sized,
//    {
//        self.base
//            .chain_read_line(overrider, Some(self.completer), suggester)
//    }
//}

//impl<S, B> LineReader for WithSuggester<'_, S, B>
//where
//    S: Suggester + ?Sized,
//    B: Builder + ChainedLineReader,
//{
//    fn read_line(self) -> Result<Outcome, crate::ErrorKind> {
//        self.base
//            .chain_read_line::<__Dummy, __Dummy, S>(None, None, Some(self.suggester))
//    }
//}

//impl<S, B> ChainedLineReader for WithSuggester<'_, S, B>
//where
//    S: Suggester + ?Sized,
//    B: Builder + ChainedLineReader,
//{
//    fn chain_read_line<'o, 'c, 'r, O, C, R>(
//        self,
//        overrider: Option<&'o O>,
//        completer: Option<&'c C>,
//        _: Option<&'r R>,
//    ) -> Result<Outcome, crate::ErrorKind>
//    where
//        O: Overrider + ?Sized,
//        C: Completer + ?Sized,
//        R: Suggester + ?Sized,
//    {
//        self.base
//            .chain_read_line(overrider, completer, Some(self.suggester))
//    }
//}

////impl Builder<Base> {
////    pub fn new() -> Self {
////        Builder(Base {
////            prompt: None,
////            buffer: None,
////            erase_after_read: false,
////        })
////    }
////}

////impl<B: BaseLineReader> Builder<B> {
////}

/////// Represents and stores a prompt that shall be presented to the user for input.
///////
/////// When built, the prompt will have no customization or completions. Also the default
/////// [`erase_after_read`] is `true`.
///////
/////// [`erase_after_read`]: struct.Prompt.html#method.erase_after_read
////pub struct Builder<'o, 'c, 's, O, C, S>
////where
////    O: Overrider + ?Sized,
////    C: Completer + ?Sized,
////    S: Suggester + ?Sized,
////{
////    prompt: Option<String>,
////    buffer: Option<Buffer>,
////    overrider: Option<&'o O>,
////    completer: Option<&'c C>,
////    suggester: Option<&'s S>,
////    erase_after_read: bool,
////}

////impl<'o, 'c, 's, O, C, S> Builder<'o, 'c, 's, O, C, S>
////where
////    O: Overrider + ?Sized,
////    C: Completer + ?Sized,
////    S: Suggester + ?Sized,
////{
////    pub fn new() -> Self {
////        Self {
////            prompt: None,
////            buffer: None,
////            overrider: None,
////            completer: None,
////            suggester: None,
////            erase_after_read: false,
////        }
////    }

////    /// Prepopulates the prompt input with `buffer`.
////    ///
////    /// # Arguments
////    ///
////    /// * [`buffer`] - A buffer to be used when displaying the prompt
////    ///
////    /// [`Buffer`]: ../buffer/trait.Buffer.html
////    #[must_use]
////    pub fn buffer(mut self, buffer: Buffer) -> Self {
////        self.buffer = Some(buffer);
////        self
////    }

////    /// Prepopulates the prompt input with `buffer`.
////    ///
////    /// # Arguments
////    ///
////    /// * [`buffer`] - A buffer to be used when displaying the prompt
////    ///
////    /// [`Buffer`]: ../buffer/trait.Buffer.html
////    #[must_use]
////    pub fn buffer(mut self, buffer: Buffer) -> Self {
////        self.buffer = Some(buffer);
////        self
////    }

////    /// Modifies the behavior of the prompt by setting a [`Overrider`].
////    ///
////    /// # Arguments
////    ///
////    /// * [`overrider`] - The new overrider
////    ///
////    /// [`Overrider`]: ../actions/trait.Overrider.html
////    #[must_use]
////    pub fn overrider<'no, NO: Overrider>(
////        self,
////        overrider: &'no NO,
////    ) -> Builder<'no, 'c, 's, NO, C, S> {
////        Builder {
////            prompt: self.prompt,
////            buffer: self.buffer,
////            overrider: Some(overrider),
////            completer: self.completer,
////            suggester: self.suggester,
////            erase_after_read: self.erase_after_read,
////        }
////    }

////    /// Sets the in-line completion provider.
////    ///
////    /// # Arguments
////    ///
////    /// * [`completer`] - The new completer
////    ///
////    /// [`Completer`]: ../completion/trait.Completer.html
////    #[must_use]
////    pub fn completer<'nc, NC: Completer>(
////        self,
////        completer: &'nc NC,
////    ) -> Builder<'o, 'nc, 's, O, NC, S> {
////        Builder {
////            prompt: self.prompt,
////            buffer: self.buffer,
////            overrider: self.overrider,
////            completer: Some(completer),
////            suggester: self.suggester,
////            erase_after_read: self.erase_after_read,
////        }
////    }

////    /// Sets the drop-down suggestion provider.
////    ///
////    /// # Arguments
////    ///
////    /// * [`suggester`] - The new suggester
////    ///
////    /// [`Suggester`]: ../completion/trait.Suggester.html
////    #[must_use]
////    pub fn suggester<'ns, NS: Suggester>(
////        self,
////        suggester: &'ns NS,
////    ) -> Builder<'o, 'c, 'ns, O, C, NS> {
////        Builder {
////            prompt: self.prompt,
////            buffer: self.buffer,
////            overrider: self.overrider,
////            completer: self.completer,
////            suggester: Some(suggester),
////            erase_after_read: self.erase_after_read,
////        }
////    }

////    /// Controls if the prompt shall be erased after user input.
////    ///
////    /// If set to `false` (default), after user input, the terminal will receive a new line
////    /// after the prompt text and the user input. Any drop-down completions will be removed,
////    /// however.
////    ///
////    /// If set to `true`, the whole prompt and input will be erased. The cursor returns to the
////    /// original position as if nothing happened.
////    pub fn erase_after_read(mut self, erase_after_read: bool) -> Self {
////        self.erase_after_read = erase_after_read;
////        self
////    }
////}

//// impl<O, C, S> LineReader for Builder<'_, '_, '_, O, C, S>
//// where
////     O: Overrider + ?Sized,
////     C: Completer + ?Sized,
////     S: Suggester + ?Sized,
//// {
////     fn read_line(self) -> Result<Outcome, crate::ErrorKind> {
////         super::read_line(
////             self.prompt.as_deref(),
////             self.buffer,
////             self.overrider,
////             self.completer,
////             self.suggester,
////             self.erase_after_read,
////         )
////     }
//// }

//// impl<S: ToString> std::convert::From<S> for Builder<Base> {
////     fn from(string: S) -> Self {
////         Builder(Base {
////             prompt: Some(string.to_string()),
////             buffer: None,
////             erase_after_read: false,
////         })
////     }
//// }

//struct __Dummy;

//impl Overrider for __Dummy {
//    fn override_for(
//        &self,
//        _: crate::actions::Event,
//        _: &dyn crate::Context,
//    ) -> Option<crate::actions::Action> {
//        unimplemented!()
//    }
//}

//impl Completer for __Dummy {
//    fn complete_for(&self, _: &dyn crate::Context) -> Option<&str> {
//        unimplemented!()
//    }
//}

//impl Suggester for __Dummy {
//    fn suggest_for(&self, _: &dyn crate::Context) -> Vec<&str> {
//        unimplemented!()
//    }
//}
