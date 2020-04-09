fn main() {
    if let Ok(Some(string)) = rucline::prompt::Prompt::new()
        .prompt(Some("abc> "))
        .completions(Some(&["some programmer was here", "some developer was there", "exit"]))
        .read_line()
    {
        println!("Got: {}", string);
    }
}
