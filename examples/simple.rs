fn main() {
    if let Some(string) = rucline::prompt::Prompt::new()
        .prompt(Some("abc> "))
        .read_line()
    {
        println!("Got: {}", string);
    }
}
