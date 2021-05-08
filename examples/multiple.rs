use rucline::crossterm::style::Colorize;
use rucline::prompt::{Builder, Prompt};
use rucline::{Buffer, Outcome};

fn main() {
    // Simulate a list of acceptable inputs
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

    let mut buffer = Buffer::new();

    // Initial prompt
    while let Ok(Outcome::Accepted(command)) = Prompt::from(format!("{}> ", "vai".dark_green()))
        .erase_after_read(true)
        .buffer(buffer.clone())
        .suggester_ref(&possible_commands)
        .completer_ref(&command_history)
        .read_line()
    {
        // Accept command if it exists
        if possible_commands.contains(&command.as_str()) {
            // Show the sub-prompt
            if let Ok(outcome) = Prompt::from(format!(
                "{}|{}> ",
                "vai".dark_green(),
                command.as_str().green()
            ))
            .erase_after_read(true)
            .completer_ref(&mode_history)
            .read_line()
            {
                match outcome {
                    Outcome::Accepted(mode) => {
                        // We will do as commanded
                        println!("Ok! Will {} {}", command, mode);
                        break;
                    }
                    Outcome::Canceled(_) => {
                        buffer = command.into();
                    }
                }
            }
        } else {
            // Command not recognized
            eprintln!("\n{} invalid command", "Error".red());
        }
    }
}
