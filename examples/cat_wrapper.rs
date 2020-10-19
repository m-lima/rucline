#![cfg(unix)]
use rucline::crossterm::style::Colorize;
use rucline::prompt::{Builder, Prompt};
use rucline::Outcome::Accepted;

use pwner::Spawner;
use std::io::{Read, Write};
use std::process::Command;

fn main() {
    // Start cat
    let mut cat = Command::new("cat")
        .spawn_owned()
        .expect("Could not start 'cat'");

    // Prepare a buffer to read from cat
    let mut buffer = [0_u8; 1024];

    // While there is some data read
    while let Ok(Accepted(input)) = Prompt::from("cat> ".green())
        .completer(vec!["quit"])
        .read_line()
    {
        // If the user wants to quit, do so
        if &input == "quit" {
            break;
        }

        // Write the line into cat
        cat.write_all(input.as_bytes())
            .expect("Could not write to 'cat'");
        cat.write_all(&[b'\n']).expect("Could not flush 'cat'");

        // Read from cat and print
        let bytes = cat.read(&mut buffer).expect("Could not read from 'cat'");
        if let Ok(string) = std::str::from_utf8(&buffer[..bytes - 1]) {
            println!("{}", string);
        }
    }
}
