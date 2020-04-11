#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub type KeyBindings = std::collections::HashMap<Event, Action>;

pub type Event = crossterm::event::KeyEvent;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Action {
    Write(char),
    Delete(Scope),
    Move(Range, Direction),
    Suggest(Direction),
    Complete(Range),
    Accept,
    Cancel,
    Noop,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Scope {
    WholeLine,
    WholeWord,
    Relative(Range, Direction),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Range {
    Line,
    Word,
    Single,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Direction {
    Forward,
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

// TODO: cannot paste multiline (it will trigger an Enter
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
