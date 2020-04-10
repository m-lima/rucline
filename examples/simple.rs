use rucline::completer::Basic;
use rucline::Prompt;

fn main() {
    if let Ok(Some(string)) = Prompt::new()
        .prompt("simple> ")
        .erase_after_read(true)
        .completer(Basic::new(&[
            "some programmer was here",
            "some developer was there",
            "exit",
        ]))
        .read_line()
    {
        println!("Got: {}", string);
    }
}
