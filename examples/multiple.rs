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

    let prompt = Prompt::new()
        .erase_after_read(true)
        .prompt(&format!("{}> ", "vai".green()))
        .suggester(Basic::new(&possible_commands))
        .completer(Basic::new(&command_history));

    let command = if let Ok(Some(command)) = prompt.read_line() {
        if possible_commands.contains(&command.as_str()) {
            Some(command)
        } else {
            eprintln!("{} invalid command", "Error".red());
            None
        }
    } else {
        None
    };

    if let Some(command) = command {
        if let Ok(Some(mode)) = Prompt::new()
            .prompt(&format!(
                "{}|{}> ",
                "vai".green(),
                command.as_str().bright_green()
            ))
            .completer(Basic::new(&mode_history))
            .read_line()
        {
            println!("Ok! Will {} {}", command, mode);
        }
    }
}
