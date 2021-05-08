use super::{Buffer, Writer};
use crate::Error;

// TODO: Keep track of lines
// TODO: Deal with colors
pub struct Crossterm {
    erase_on_drop: bool,
    prompt_length: usize,
    printed_length: usize,
    cursor_offset: usize,
}

impl Crossterm {
    pub fn new(erase_on_drop: bool) -> Self {
        Self {
            erase_on_drop,
            prompt_length: 0,
            printed_length: 0,
            cursor_offset: 0,
        }
    }
}

impl Writer for Crossterm {
    type Error = Error;

    fn begin(&mut self, prompt: Option<&str>) -> Result<(), Error> {
        crossterm::terminal::enable_raw_mode()?;
        if let Some(prompt) = prompt {
            use std::io::Write;

            self.prompt_length +=
                unicode_segmentation::UnicodeSegmentation::graphemes(prompt, true).count();
            crossterm::execute!(std::io::stdout(), crossterm::style::Print(prompt))
        } else {
            Ok(())
        }
    }

    fn print(&mut self, buffer: &Buffer, completion: Option<&str>) -> Result<(), Error> {
        use std::io::Write;
        use unicode_segmentation::UnicodeSegmentation;

        let mut stdout = std::io::stdout();

        clear_from(&mut stdout, self.printed_length - self.cursor_offset)?;

        self.printed_length = buffer.graphemes(true).count();
        self.cursor_offset =
            self.printed_length - buffer[0..buffer.cursor()].graphemes(true).count();

        crossterm::queue!(&mut stdout, crossterm::style::Print(&buffer))?;

        if let Some(completion) = completion {
            use crossterm::style::Colorize;

            let completion_len = completion.graphemes(true).count();
            crossterm::queue!(
                &mut stdout,
                crossterm::style::PrintStyledContent(crossterm::style::style(completion).blue())
            )?;
            rewind_cursor(&mut stdout, completion_len)?;
        }

        rewind_cursor(&mut stdout, self.cursor_offset)?;
        crossterm::execute!(&mut stdout)
    }

    fn print_suggestions(
        &mut self,
        selected_index: usize,
        suggestions: &[std::borrow::Cow<'_, str>],
    ) -> Result<(), Error> {
        use std::io::Write;
        use unicode_segmentation::UnicodeSegmentation;

        let mut stdout = std::io::stdout();

        // Print buffer
        let buffer = suggestions[selected_index].as_ref();
        clear_from(&mut stdout, self.printed_length - self.cursor_offset)?;
        crossterm::queue!(stdout, crossterm::style::Print(buffer))?;
        self.cursor_offset = 0;
        self.printed_length = buffer.graphemes(true).count();

        // Save position at the end of the buffer
        // TODO: avoid this save and the later restore
        let end_of_buffer = crossterm::cursor::position().map(|pos| pos.0)?;

        // Print suggestions
        for (index, suggestion) in suggestions.iter().enumerate() {
            if index == selected_index {
                use crossterm::style::Styler;
                crossterm::queue!(
                    stdout,
                    crossterm::style::Print('\n'),
                    crossterm::cursor::MoveToColumn(0),
                    crossterm::style::PrintStyledContent(
                        crossterm::style::style(suggestion).bold()
                    ),
                )?;
            } else {
                crossterm::queue!(
                    stdout,
                    crossterm::style::Print('\n'),
                    crossterm::cursor::MoveToColumn(0),
                    crossterm::style::Print(suggestion),
                )?;
            }
        }

        // Rewind suggestions cursor
        for suggestion in suggestions.iter().rev() {
            let length = suggestion.graphemes(true).count();
            rewind_cursor(&mut stdout, length)?;
            crossterm::queue!(stdout, crossterm::cursor::MoveUp(1))?;
        }

        // Restore cursor
        let bottom_of_buffer = crossterm::cursor::position().map(|pos| pos.1)?;
        crossterm::queue!(
            stdout,
            crossterm::cursor::MoveTo(end_of_buffer, bottom_of_buffer)
        )?;
        rewind_cursor(&mut stdout, self.cursor_offset)?;

        // Execute
        crossterm::execute!(stdout)
    }
}

fn clear_from(stdout: &mut std::io::Stdout, amount: usize) -> Result<(), Error> {
    use std::io::Write;

    rewind_cursor(stdout, amount)?;

    crossterm::queue!(
        stdout,
        crossterm::terminal::Clear(crossterm::terminal::ClearType::FromCursorDown),
    )
}

// Allowed because we slice `usize` into `u16` chunks
#[allow(clippy::cast_possible_truncation)]
fn rewind_cursor(stdout: &mut std::io::Stdout, amount: usize) -> Result<(), Error> {
    use std::io::Write;

    if amount == 0 {
        return Ok(());
    }

    let mut remaining = amount;
    while remaining > usize::from(u16::max_value()) {
        crossterm::queue!(stdout, crossterm::cursor::MoveLeft(u16::max_value()))?;
        remaining -= usize::from(u16::max_value());
    }

    crossterm::queue!(stdout, crossterm::cursor::MoveLeft(remaining as u16))
}

// Allowed because we slice `usize` into `u16` chunks
#[allow(clippy::cast_possible_truncation)]
fn fast_forward_cursor(stdout: &mut std::io::Stdout, amount: usize) -> Result<(), Error> {
    use std::io::Write;

    if amount == 0 {
        return Ok(());
    }

    let mut remaining = amount;
    while remaining > usize::from(u16::max_value()) {
        crossterm::queue!(stdout, crossterm::cursor::MoveRight(u16::max_value()))?;
        remaining -= usize::from(u16::max_value());
    }

    crossterm::queue!(stdout, crossterm::cursor::MoveRight(remaining as u16))
}

impl std::ops::Drop for Crossterm {
    // Allowed because this is a drop and the previous construction already managed the get through
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        use std::io::Write;
        crossterm::terminal::disable_raw_mode();

        let mut stdout = std::io::stdout();

        if self.erase_on_drop {
            clear_from(&mut stdout, self.prompt_length + self.printed_length);
        } else {
            fast_forward_cursor(&mut stdout, self.cursor_offset);
            crossterm::execute!(
                stdout,
                crossterm::terminal::Clear(crossterm::terminal::ClearType::FromCursorDown),
                crossterm::style::Print('\n')
            );
        }
    }
}
