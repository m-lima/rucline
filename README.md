# rucline
[![Github](https://github.com/m-lima/rucline/workflows/build/badge.svg)](https://github.com/m-lima/rucline/actions?workflow=build)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Cargo](https://img.shields.io/crates/v/rucline.svg)](https://crates.io/crates/rucline)
[![Documentation](https://docs.rs/rucline/badge.svg)](https://docs.rs/rucline)

![demo](docs/demo.gif)

Rucline (Rust CLI line /rɪˈklaɪn/) is a cross-platform, UTF-8 aware, advanced edtigin,
autocompletion capable, tab suggestion supporting line reader you can "recline" on.

It provides advanced editing **actions** for user input and customization of the line
reader.

It uses **crossterm** as a backend to provide cross-platform support, and provides advanced

#### Basic usage:

```rust
use rucline::completion;
use rucline::Prompt;

if let Ok(Some(string)) = Prompt::new()
    // Create a bold prompt
    .prompt("What's you favorite website? ")
    // Add some likely values as completions
    .completer(completion::Basic::new(&[
        "https://www.rust-lang.org/",
        "https://docs.rs/",
        "https://crates.io/",
    ]))
    // Add some tab completions
    .suggester(completion::Basic::new(&[
        "https://www.startpage.com/",
        "https://www.google.com/",
    ]))
    //Block until value is ready
    .read_line()
{
    println!("'{}' seems to be your favorite website", string);
}
```

## Actions

Rucline allow advanced **actions** for interacting with the **Prompt**, but it
comes built-in with useful behavior. For example, a few of the build-ins:
* `Tab`: cycle through completions
* `Shift` + `Tab`: cycle through completions in reverse
* `CTRL` + `W`: delete the current work
* `CTRL` + `J`: delete the beginning of the word
* `CTRL` + `K`: delete the end of the word
* `CTRL` + `U`: delete the line
* `CTRL` + `H`: delete the beggining of the line
* `CTRL` + `L`: delete the end of the line

**See `Action` for the full default behavior**

The behavior can be customized by overriding user **events** with **actions**. Which
in turn can be serialized, stored, and loaded at run-time.


#### Overriding key bindings

```rust
use rucline::{completion, Prompt};
use rucline::key_bindings::{Action, Event, KeyBindings, Range};
use crossterm::event::KeyCode;

let mut bindings = KeyBindings::new();

// Accept the full suggestions if `right` is pressed
bindings.insert(Event::from(KeyCode::Right), Action::Complete(Range::Line));

if let Ok(Some(string)) = Prompt::new()
    // Create a bold prompt
    .prompt("What's you favorite website? ")
    // Add some likely values as completions
    .completer(completion::Basic::new(&[
        "https://www.rust-lang.org/",
        "https://docs.rs/",
        "https://crates.io/",
    ]))
    // Override the `right` key to always fill the full suggestions line
    .bindings(bindings)
    //Block until value is ready
    .read_line()
{
    println!("'{}' seems to be your favorite website", string);
}
```
