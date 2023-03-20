use crossterm::style::Stylize;
use rucline::prompt::{Builder, Prompt};
use rucline::Outcome::Accepted;

fn main() {
    if let Ok(Accepted(string)) = Prompt::from("What's you favorite website? ".bold())
        // Add some likely values as completions
        .completer(vec![
            "https://www.rust-lang.org/",
            "https://docs.rs/",
            "https://crates.io/",
        ])
        // Add some tab completions
        .suggester(vec![
            "https://www.startpage.com/",
            "https://www.google.com/",
        ])
        //Block until value is ready
        .read_line()
    {
        println!("'{string}' seems to be your favorite website");
    }
}
