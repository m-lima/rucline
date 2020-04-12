use colored::Colorize;
use rucline::completion::Basic;
use rucline::Prompt;

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

    // Initial prompt
    let prompt = Prompt::new()
        .erase_after_read(true)
        .prompt(&format!("{}> ", "vai".green()))
        .suggester(Basic::new(&possible_commands))
        .completer(Basic::new(&command_history));

    if let Ok(Some(command)) = prompt.read_line() {
        if possible_commands.contains(&command.as_str()) {
            // Accept command and show sub prompt
            if let Ok(Some(mode)) = Prompt::new()
                .prompt(&format!(
                    "{}|{}> ",
                    "vai".green(),
                    command.as_str().bright_green()
                ))
                .completer(Basic::new(&mode_history))
                .read_line()
            {
                // We will do as commanded
                println!("Ok! Will {} {}", command, mode);
            }
        } else {
            // Command not recognized
            eprintln!("{} invalid command", "Error".red());
        }
    }
}
