use crossterm::event::{KeyCode, KeyModifiers};
use crossterm::style::Stylize;
use rucline::actions::{Action, Event, Overrider};
use rucline::prompt::{Builder, Prompt};
use rucline::Buffer;
use rucline::Outcome::Accepted;

struct EarlyExit;

// An overrider that captures CTRL+D and exits the application
impl Overrider for EarlyExit {
    fn override_for(&self, event: Event, _: &Buffer) -> Option<Action> {
        if event.modifiers == KeyModifiers::CONTROL {
            if let KeyCode::Char('d') = event.code {
                // Cleanly exit when the combination CTRL+D is pressed
                quit::with_code(0);
            }
        }

        // Fallback to default action
        None
    }
}

#[quit::main]
fn main() {
    // Simulate possible commands
    let possible_commands = vec!["run", "walk", "fly"];

    // Simulate a history of previous inputs
    let command_history = vec!["run", "fly"];
    let mode_history = vec![
        "slow",
        "fast",
        "normal",
        "very slowly almost stopping",
        "very quickly almost lightspeed",
    ];

    // Initial prompt
    if let Ok(Accepted(command)) = Prompt::from(format!("{}> ", "vai".green()))
        .erase_after_read(true)
        .suggester_ref(&possible_commands)
        .completer(command_history)
        .overrider_ref(&EarlyExit)
        .read_line()
    {
        // Accept command if it exists
        if possible_commands.contains(&command.as_str()) {
            // Show the sub-prompt
            if let Ok(Accepted(mode)) = Prompt::from(format!(
                "{}|{}> ",
                "vai".dark_green(),
                command.as_str().green()
            ))
            .completer(mode_history)
            .overrider_ref(&EarlyExit)
            .read_line()
            {
                // We will do as commanded
                println!("Ok! Will {command} {mode}");
            }
        } else {
            // Command not recognized
            eprintln!("{} invalid command", "Error".red());
        }
    }
}
