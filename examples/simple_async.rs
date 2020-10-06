use colored::Colorize;
use rucline::completion::Basic;
use rucline::Prompt;

#[tokio::main]
async fn main() {
    let completer = Basic::new(&[
        "https://www.rust-lang.org/",
        "https://docs.rs/",
        "https://crates.io/",
    ]);

    let suggester = Basic::new(&["https://www.startpage.com/", "https://www.google.com/"]);

    let prompt = Prompt::from("[10s] What's you favorite website? ".bold())
        .completer(&completer)
        .suggester(&suggester);

    tokio::select! {
        line = prompt.read_line_async() => {
            if let Ok(Some(string)) = line {
                println!("'{}' seems to be your favorite website", string);
            }
        }
    }
}
