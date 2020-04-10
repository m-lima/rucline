fn main() {
    if let Ok(Some(string)) = rucline::prompt::Prompt::new()
        .prompt(Some("abc> "))
        .completions(Some(&["some programmer was here", "some developer was there", "exit", "zzzzzzzzzzzzzzzzzzzlkja sdlkfja ldskfj alksdjfal ksdjf alkdsjf lakdjsf lakjsd flkajsd lkja sdlkfj alksdjf alkjsdf ;lakjsd flkjahsd lfkjha sldfkjha lsdkjfh alkjsdhf lakjhdf lakjhdsf lakjhdsf lakjhdsf lakjhds flkajhds flakjhsdf"]))
        .read_line()
    {
        println!("Got: {}", string);
    }
}
