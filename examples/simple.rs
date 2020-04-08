fn main() {
    if let Ok(Some(string)) = rucline::prompt::Prompt::new()
        .prompt(Some("abc> "))
        .read_line()
    {
        println!("Got: {}", string);
    }
}
