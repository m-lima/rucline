use colored::Colorize;
use crossterm::event::KeyCode;
use rucline::actions::{Action, Event, KeyBindings, Range};
use rucline::{completion, Prompt};

fn main() {
    let mut bindings = KeyBindings::new();

    // Accept the full suggestions if `right` is pressed
    bindings.insert(Event::from(KeyCode::Right), Action::Complete(Range::Line));

    if let Ok(Some(string)) = Prompt::from("What's you favorite website? ".bold())
        // Add some likely values as completions
        .completer(&completion::Basic::new(&[
            "https://www.rust-lang.org/",
            "https://docs.rs/",
            "https://crates.io/",
        ]))
        // Set the new key bindings as an override
        .overrider(&bindings)
        //Block until value is ready
        .read_line()
    {
        println!("'{}' seems to be your favorite website", string);
    }
}
