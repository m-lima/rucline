use rucline::completion;
use rucline::Prompt;

fn main() {
    if let Ok(Some(string)) = Prompt::new()
        .prompt("simple> ")
        .erase_after_read(true)
        .completer(completion::Basic::new(&[
            "some programmer was here",
            "some developer was there",
        ]))
        .suggester(completion::Basic::new(&[
            "one suggestions here",
            "another one over there",
            "exit",
        ]))
        .read_line()
    {
        println!("Got: {}", string);
    }
}
