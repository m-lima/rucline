//! Provides mappings and actions to change the behavior of the [`prompt`] when reading user input.
//!
//! There is a built-in set of default [`Action`]s that will be executed upon user interaction.
//! These are meant to feel natural when coming from the default terminal, while also adding further
//! functionality and editing commands.
//!
//! However, bindings that override the default behavior can be given to the [`prompt`] to cause
//! a different [`Action`] to be taken.
//!
//! # Examples
//!
//! Changing the behavior of `TAB` from the default [`Suggest`] action to actually printing `\t`.
//!
//! Using [`KeyBindings`]:
//! ```
//! use rucline::actions::{Action, Event, KeyBindings, KeyCode};
//! use rucline::prompt::{Builder, Prompt};
//!
//! let mut bindings = KeyBindings::new();
//! bindings.insert(Event::from(KeyCode::Tab), Action::Write('\t'));
//!
//! let prompt = Prompt::new().overrider(bindings);
//! ```
//!
//! Using a closure:
//! ```
//! use rucline::actions::{Action, Event, KeyCode};
//! use rucline::prompt::{Builder, Prompt};
//!
//! let prompt = Prompt::new().overrider_fn(|e, _| {
//!     if e == Event::from(KeyCode::Tab) {
//!         Some(Action::Write('\t'))
//!     } else {
//!         None
//!     }
//! });
//! ```
//!
//! # Overriding a default action
//!
//! The [`KeyBindings`] map provides a way to store the association
//! between [`Event`] and [`Action`] which will override the default behavior.
//!
//! ```
//! use rucline::actions::{Action, Event, KeyBindings, KeyCode};
//!
//! let mut bindings = KeyBindings::new();
//! bindings.insert(Event::from(KeyCode::Tab), Action::Write('\t'));
//! ```
//!
//! # Disabling a default action
//!
//! To explicitly remove an [`Action`] from the default behavior, the [`NoOp`] action can be
//! set as the override.
//!
//! ```
//! use rucline::actions::{Action, Event, KeyBindings, KeyCode};
//!
//! let mut bindings = KeyBindings::new();
//! bindings.insert(Event::from(KeyCode::Tab), Action::NoOp);
//! ```
//!
//! # Saving key binding configurations
//!
//! If the feature `config-serde` is enabled, [`KeyBindings`] can be serialized, stored, and loaded
//! at runtime.
//!
//! # Default behavior
//!
//! In the absence of [`KeyBindings`] or an entry for a given [`Event`], the default behavior
//! will be as follows:
//!
//! ```no_run
//! # fn default_action(event: rucline::actions::Event) -> rucline::actions::Action {
//! # use rucline::actions::{Action::*, Direction::*, KeyCode, Range::*, Scope::* };
//! # match event.code {
//! KeyCode::Enter => Accept,
//! KeyCode::Esc => Cancel,
//! KeyCode::Tab => Suggest(Forward),
//! KeyCode::BackTab => Suggest(Backward),
//! KeyCode::Backspace => Delete(Relative(Single, Backward)),
//! KeyCode::Delete => Delete(Relative(Single, Forward)),
//! KeyCode::Right => Move(Single, Forward),
//! KeyCode::Left => Move(Single, Backward),
//! KeyCode::Home => Move(Line, Backward),
//! KeyCode::End => Move(Line, Forward),
//! KeyCode::Char(c) => {
//!     if event.modifiers == crossterm::event::KeyModifiers::CONTROL {
//!         match c {
//!             'm' | 'd' => Accept,
//!             'c' => Cancel,
//!
//!             'b' => Move(Single, Backward),
//!             'f' => Move(Single, Forward),
//!             'a' => Move(Line, Backward),
//!             'e' => Move(Line, Forward),
//!
//!             'j' => Delete(Relative(Word, Backward)),
//!             'k' => Delete(Relative(Word, Forward)),
//!             'h' => Delete(Relative(Line, Backward)),
//!             'l' => Delete(Relative(Line, Forward)),
//!             'w' => Delete(WholeWord),
//!             'u' => Delete(WholeLine),
//!             _ => NoOp,
//!         }
//!     } else if event.modifiers == crossterm::event::KeyModifiers::ALT {
//!         match c {
//!             'b' => Move(Word, Backward),
//!             'f' => Move(Word, Forward),
//!             _ => NoOp,
//!         }
//!     } else {
//!         Write(c)
//!     }
//! }
//! _ => NoOp,
//! # }}
//! ```
//!
//!  > Check the test cases for [`Buffer`] to see how line edits are expected to behave.
//!
//! [`Action`]: enum.Action.html
//! [`Event`]: type.Event.html
//! [`KeyBindings`]: type.KeyBindings.html
//! [`NoOp`]: enum.Action.html#variant.NoOp
//! [`Suggest`]: enum.Action.html#variant.Suggest
//! [`prompt`]: ../prompt/index.html
//! [`Buffer`]: ../buffer/struct.Buffer.html

use crate::Buffer;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Alias to `crossterm::event::KeyEvent` from [`crossterm`](https://docs.rs/crossterm/).
pub use crossterm::event::KeyEvent as Event;

/// Alias to `crossterm::event::KeyCode` from [`crossterm`](https://docs.rs/crossterm/).
pub use crossterm::event::KeyCode;

/// Alias to [`HashMap<Event, Action>`](std::collections::HashMap)
pub type KeyBindings = std::collections::HashMap<Event, Action>;

/// An action that can be performed while reading a line
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Action {
    /// Write a single character where the cursor is
    Write(char),
    /// Delete a section based on the cursor, defined by [`Scope`](enum.Scope.html)
    Delete(Scope),
    /// Move the cursor for a [`Range`](enum.Range.html) in a [`Direction`](enum.Direction.html)
    Move(Range, Direction),
    /// Trigger the [`suggester`](../completion/trait.Suggester.html)
    Suggest(Direction),
    /// Accept [`Range`](enum.Range.html) from the current completion presented by
    /// [`completer`](../completion/trait.Completer.html), if any
    Complete(Range),
    /// Accept the current line
    Accept,
    /// Cancel the suggestions, if any. Else, discard the whole line
    Cancel,
    /// Do nothing and wait for the next [`Event`](type.Event.html)
    NoOp,
}

/// The scope an [`Action`](enum.Action.html) should be applied on
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Scope {
    /// Represents a whole line
    WholeLine,
    /// Represents a whole word
    WholeWord,
    /// Represents a relative scope, with a [`Range`](enum.Range.html)
    /// and [`Direction`](enum.Direction.html)
    Relative(Range, Direction),
}

/// The range an [`Action`](enum.Action.html) should extend for
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Range {
    /// Represents the remainder of the line
    Line,
    /// Represents a single word
    Word,
    /// Represents a single character
    Single,
}

/// The direction an [`Action`](enum.Action.html) may take
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Direction {
    /// Represents a "right" or "down" direction
    Forward,
    /// Represents a "left" or "up" direction
    Backward,
}

/// Overrides the behavior for a given [`Event`].
///
/// This trait has a convenience implementation for [`KeyBindings`] and also a conversion
/// from closures.
///
/// # Example
///
/// ```
/// use rucline::actions::{Action, Event, KeyCode};
/// use rucline::prompt::{Builder, Prompt};
///
/// let prompt = Prompt::new().overrider_fn(|e, _| if e == Event::from(KeyCode::Tab) {
///     Some(Action::Write('\t'))
/// } else {
///     None
/// });
/// ```
///
/// [`Event`]: type.Event.html
/// [`KeyBindings`]: type.KeyBindings.html
pub trait Overrider {
    /// Overrides the behavior for the given [`Event`].
    ///
    /// [`Buffer`] will contain the current context of the prompt, in case the behavior should
    /// be contextual.
    ///
    /// # Arguments
    /// * [`event`] - The incoming event to be processed.
    /// * [`buffer`] - The current context in which this event is coming in.
    ///
    /// [`Event`]: type.Event.html
    /// [`Buffer`]: ../buffer/struct.Buffer.html
    fn override_for(&self, event: Event, buffer: &Buffer) -> Option<Action>;
}

impl Overrider for KeyBindings {
    fn override_for(&self, event: Event, _: &Buffer) -> Option<Action> {
        self.get(&event).copied()
    }
}

impl<F> Overrider for F
where
    F: Fn(Event, &Buffer) -> Option<Action>,
{
    fn override_for(&self, event: Event, buffer: &Buffer) -> Option<Action> {
        self(event, buffer)
    }
}

pub(super) fn action_for<O: Overrider + ?Sized>(
    overrider: Option<&O>,
    event: Event,
    buffer: &Buffer,
) -> Action {
    overrider
        .as_ref()
        .and_then(|b| b.override_for(event, buffer))
        .unwrap_or_else(|| default_action(event, buffer))
}

#[inline]
fn control_pressed(event: &Event) -> bool {
    event.modifiers == crossterm::event::KeyModifiers::CONTROL
}

#[inline]
fn alt_pressed(event: &Event) -> bool {
    event.modifiers == crossterm::event::KeyModifiers::ALT
}

#[inline]
fn complete_if_at_end_else_move(buffer: &Buffer, range: Range) -> Action {
    if buffer.cursor() == buffer.len() {
        if range == Range::Word {
            Action::Complete(Range::Word)
        } else {
            Action::Complete(Range::Line)
        }
    } else {
        Action::Move(range, Direction::Forward)
    }
}

fn default_action(event: Event, buffer: &Buffer) -> Action {
    use Action::{Accept, Cancel, Delete, Move, NoOp, Suggest, Write};
    use Direction::{Backward, Forward};
    use Range::{Line, Single, Word};
    use Scope::{Relative, WholeLine, WholeWord};

    match event.code {
        KeyCode::Enter => Accept,
        KeyCode::Esc => Cancel,
        KeyCode::Tab => Suggest(Forward),
        KeyCode::BackTab => Suggest(Backward),
        KeyCode::Backspace => Delete(Relative(Single, Backward)),
        KeyCode::Delete => Delete(Relative(Single, Forward)),
        KeyCode::Right => complete_if_at_end_else_move(buffer, Single),
        KeyCode::Left => Move(Single, Backward),
        KeyCode::Home => Move(Line, Backward),
        KeyCode::End => complete_if_at_end_else_move(buffer, Line),
        KeyCode::Char(c) => {
            if control_pressed(&event) {
                match c {
                    'm' | 'd' => Accept,
                    'c' => Cancel,

                    'b' => Move(Single, Backward),
                    'f' => complete_if_at_end_else_move(buffer, Single),
                    'a' => Move(Line, Backward),
                    'e' => complete_if_at_end_else_move(buffer, Line),

                    'j' => Delete(Relative(Word, Backward)),
                    'k' => Delete(Relative(Word, Forward)),
                    'h' => Delete(Relative(Line, Backward)),
                    'l' => Delete(Relative(Line, Forward)),
                    'w' => Delete(WholeWord),
                    'u' => Delete(WholeLine),
                    _ => NoOp,
                }
            } else if alt_pressed(&event) {
                match c {
                    'b' => Move(Word, Backward),
                    'f' => complete_if_at_end_else_move(buffer, Word),
                    _ => NoOp,
                }
            } else {
                Write(c)
            }
        }
        _ => NoOp,
    }
}

#[cfg(test)]
mod test {
    use super::{action_for, default_action, Action, Buffer, Direction, Event, KeyCode, Range};

    #[test]
    fn should_complete_if_at_end() {
        use crossterm::event::KeyModifiers;
        use Action::{Complete, Move};
        use Direction::Forward;
        use KeyCode::{Char, End, Right};
        use Range::{Line, Single, Word};

        let mut c = "a".into();

        assert_eq!(default_action(Event::from(Right), &c), Complete(Line));
        assert_eq!(default_action(Event::from(End), &c), Complete(Line));
        assert_eq!(
            default_action(Event::new(Char('f'), KeyModifiers::CONTROL), &c),
            Complete(Line)
        );
        assert_eq!(
            default_action(Event::new(Char('f'), KeyModifiers::ALT), &c),
            Complete(Word)
        );

        c.set_cursor(0).unwrap();

        assert_eq!(
            default_action(Event::from(Right), &c),
            Move(Single, Forward)
        );
        assert_eq!(default_action(Event::from(End), &c), Move(Line, Forward));
        assert_eq!(
            default_action(Event::new(Char('f'), KeyModifiers::CONTROL), &c),
            Move(Single, Forward)
        );
        assert_eq!(
            default_action(Event::new(Char('f'), KeyModifiers::ALT), &c),
            Move(Word, Forward)
        );
    }

    #[test]
    fn should_default_if_no_mapping() {
        use super::KeyBindings;
        use KeyCode::Tab;
        let action = action_for::<KeyBindings>(None, Event::from(Tab), &Buffer::new());
        assert_eq!(action, Action::Suggest(Direction::Forward));
    }

    mod basic {
        use super::super::{
            action_for, Action, Buffer, Direction, Event, KeyBindings, KeyCode::Tab,
        };

        #[test]
        fn should_default_if_event_missing_form_mapping() {
            let overrider = KeyBindings::new();
            let action = action_for(Some(&overrider), Event::from(Tab), &Buffer::new());
            assert_eq!(action, Action::Suggest(Direction::Forward));
        }

        #[test]
        fn should_override_if_defined() {
            let mut bindings = KeyBindings::new();
            bindings.insert(Event::from(Tab), Action::Write('\t'));
            let action = action_for(Some(&bindings), Event::from(Tab), &Buffer::new());
            assert_eq!(action, Action::Write('\t'));
        }
    }

    mod closure {
        use super::super::{action_for, Action, Buffer, Direction, Event, KeyCode::Tab};

        #[test]
        fn should_default_if_event_missing_form_mapping() {
            let overrider = |_, _: &Buffer| None;
            let action = action_for(Some(&overrider), Event::from(Tab), &Buffer::new());
            assert_eq!(action, Action::Suggest(Direction::Forward));
        }

        #[test]
        fn should_override_if_defined() {
            let overrider = |e, _: &Buffer| {
                if e == Event::from(Tab) {
                    Some(Action::Write('\t'))
                } else {
                    None
                }
            };
            let action = action_for(Some(&overrider), Event::from(Tab), &Buffer::new());
            assert_eq!(action, Action::Write('\t'));
        }
    }
}
