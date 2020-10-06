use super::Buffer;

// TODO: Keep track of lines
pub(super) struct Writer {
    erase_on_drop: Option<usize>,
    printed_length: usize,
    cursor_offset: usize,
}

impl Writer {
    pub(super) fn new(erase_on_drop: bool, prompt: Option<&str>) -> Result<Self, crate::ErrorKind> {
        crossterm::terminal::enable_raw_mode()?;
        if let Some(prompt) = prompt {
            use std::io::Write;

            crossterm::queue!(std::io::stdout(), crossterm::style::Print(prompt))?;
        }

        let erase_on_drop = if erase_on_drop {
            prompt
                .map(|s| unicode_segmentation::UnicodeSegmentation::graphemes(s, true).count())
                .or(Some(0))
        } else {
            None
        };

        Ok(Self {
            erase_on_drop,
            printed_length: 0,
            cursor_offset: 0,
        })
    }

    pub(super) fn print(
        &mut self,
        buffer: &Buffer,
        completion: Option<&str>,
    ) -> Result<(), crate::ErrorKind> {
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

    pub(super) fn print_suggestions(
        &mut self,
        selected_index: usize,
        suggestions: &[&str],
    ) -> Result<(), crate::ErrorKind> {
        use std::io::Write;
        use unicode_segmentation::UnicodeSegmentation;

        let mut stdout = std::io::stdout();

        // Print buffer
        let buffer = suggestions[selected_index];
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

fn clear_from(stdout: &mut std::io::Stdout, amount: usize) -> Result<(), crate::ErrorKind> {
    use std::io::Write;

    rewind_cursor(stdout, amount)?;

    crossterm::queue!(
        stdout,
        crossterm::terminal::Clear(crossterm::terminal::ClearType::FromCursorDown),
    )
}

// Allowed because we slice `usize` into `u16` chunks
#[allow(clippy::cast_possible_truncation)]
fn rewind_cursor(stdout: &mut std::io::Stdout, amount: usize) -> Result<(), crate::ErrorKind> {
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
fn fast_forward_cursor(
    stdout: &mut std::io::Stdout,
    amount: usize,
) -> Result<(), crate::ErrorKind> {
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

impl std::ops::Drop for Writer {
    // Allowed because this is a drop and the previous construction already managed the get through
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        use std::io::Write;
        crossterm::terminal::disable_raw_mode();

        let mut stdout = std::io::stdout();

        if let Some(prompt_length) = self.erase_on_drop {
            clear_from(&mut stdout, self.printed_length + prompt_length);
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
