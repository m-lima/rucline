// TODO: Consider making bindings also available as a lambda

//! Provides mappings and actions to change the behavior of [`Prompt`] when parsing the user input.
//!
//! There is a default set of [`Action`]s that will be executed given user inputs. These mainly
//! emulate the bahavior of a standard terminal.
//!
//! However, bindings that override the default behavior can be given to [`Prompt`] to cause
//! a different [`Action`] to be taken.
//!
//! # Example
//!
//! ```
//! use rucline::Prompt;
//! use rucline::key_bindings::{Action, Event, KeyBindings};
//! use crossterm::event::KeyCode;
//!
//! let mut bindings = KeyBindings::new();
//! bindings.insert(Event::from(KeyCode::Tab), Action::Write('\t'));
//!
//! let prompt = Prompt::new().bindings(bindings);
//! ```
//!
//! # Overriding a default action
//!
//! The [`KeyBindings`] map provides a way to store the association
//! between [`Event`] and [`Action`] which will override the default behavior.
//!
//! ```
//! use rucline::key_bindings::{Action, Event, KeyBindings};
//! use crossterm::event::KeyCode;
//!
//! let mut bindings = KeyBindings::new();
//! bindings.insert(Event::from(KeyCode::Tab), Action::Write('\t'));
//! ```
//!
//! # Disabling a default action
//!
//! To explicitly remove an [`Action`] from the default behavior, the [`Noop`] action can be
//! set as the override.
//!
//! ```
//! use rucline::key_bindings::{Action, Event, KeyBindings};
//! use crossterm::event::KeyCode;
//!
//! let mut bindings = KeyBindings::new();
//! bindings.insert(Event::from(KeyCode::Tab), Action::Noop);
//! ```
//!
//! # Saving key binding configurations
//!
//! If the feature `serialize`, [`KeyBindings`] can be serialized and stored.
//!
//!
//! # Default behavior
//!
//! In the absence of [`KeyBindings`] or an entry for a given [`Event`], the default behavior
//! will be as follows:
//!
//! ```ignore
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
//!             _ => Noop,
//!         }
//!     } else if event.modifiers == crossterm::event::KeyModifiers::ALT {
//!         match c {
//!             'b' => Move(Range::Word, Backward),
//!             'f' => Move(Range::Word, Forward),
//!             _ => Noop,
//!         }
//!     } else {
//!         Write(c)
//!     }
//! }
//! _ => Noop,
//! ```
//!
//! [`Prompt`]: ../prompt/struct.Prompt.html
//! [`KeyBindings`]: type.KeyBindings.html
//! [`Event`]: type.Event.html
//! [`Action`]: enum.Action.html
//! [`Noop`]: enum.Action.html#variant.Noop

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Alias to `crossterm::event::KeyEvent` from [`crossterm`](https://docs.rs/crossterm/)
pub type Event = crossterm::event::KeyEvent;

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
    Noop,
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
    /// Represents a "left" or "down" direction
    Forward,
    /// Represents a "right" or "up" direction
    Backward,
}

pub(super) fn action_for(
    overrides: Option<&KeyBindings>,
    event: crossterm::event::KeyEvent,
) -> Action {
    if let Some(action) = overrides.and_then(|b| b.get(&event).map(Clone::clone)) {
        action
    } else {
        default_action(event)
    }
}

// TODO: Investigate '\n' being parsed and 'ENTER'
fn default_action(event: crossterm::event::KeyEvent) -> Action {
    use crossterm::event::KeyCode;
    use Action::*;
    use Direction::*;
    use Range::*;
    use Scope::*;

    match event.code {
        KeyCode::Enter => Accept,
        KeyCode::Esc => Cancel,
        KeyCode::Tab => Suggest(Forward),
        KeyCode::BackTab => Suggest(Backward),
        KeyCode::Backspace => Delete(Relative(Single, Backward)),
        KeyCode::Delete => Delete(Relative(Single, Forward)),
        KeyCode::Right => Move(Single, Forward),
        KeyCode::Left => Move(Single, Backward),
        KeyCode::Home => Move(Line, Backward),
        KeyCode::End => Move(Line, Forward),
        KeyCode::Char(c) => {
            if event.modifiers == crossterm::event::KeyModifiers::CONTROL {
                match c {
                    'm' | 'd' => Accept,
                    'c' => Cancel,

                    'b' => Move(Single, Backward),
                    'f' => Move(Single, Forward),
                    'a' => Move(Line, Backward),
                    'e' => Move(Line, Forward),

                    'j' => Delete(Relative(Word, Backward)),
                    'k' => Delete(Relative(Word, Forward)),
                    'h' => Delete(Relative(Line, Backward)),
                    'l' => Delete(Relative(Line, Forward)),
                    'w' => Delete(WholeWord),
                    'u' => Delete(WholeLine),
                    _ => Noop,
                }
            } else if event.modifiers == crossterm::event::KeyModifiers::ALT {
                match c {
                    'b' => Move(Range::Word, Backward),
                    'f' => Move(Range::Word, Forward),
                    _ => Noop,
                }
            } else {
                Write(c)
            }
        }
        _ => Noop,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_default_if_no_mapping() {
        let action = action_for(
            None,
            crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Tab),
        );
        assert_eq!(action, Action::Suggest(Direction::Forward));
    }

    #[test]
    fn should_default_if_event_missing_form_mapping() {
        let bindings = KeyBindings::new();
        let action = action_for(
            Some(&bindings),
            crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Tab),
        );
        assert_eq!(action, Action::Suggest(Direction::Forward));
    }

    #[test]
    fn should_override_if_defined() {
        let mut bindings = KeyBindings::new();
        bindings.insert(
            crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Tab),
            Action::Write('\t'),
        );
        let action = action_for(
            Some(&bindings),
            crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Tab),
        );
        assert_eq!(action, Action::Write('\t'));
    }
}
