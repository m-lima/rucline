use super::{Buffer, CharString, CharStringView};

pub(super) struct Writer {
    erase_on_drop: Option<usize>,
    printed_length: usize,
    cursor_offset: usize,
}

impl Writer {
    pub(super) fn new(
        erase_on_drop: bool,
        prompt: Option<&CharString>,
    ) -> Result<Self, crate::ErrorKind> {
        crossterm::terminal::enable_raw_mode()?;
        if let Some(prompt) = prompt {
            use std::io::Write;

            crossterm::queue!(std::io::stdout(), crossterm::style::Print(prompt))?;
        }

        let erase_on_drop = if erase_on_drop {
            prompt.map(CharString::len).or(Some(0))
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
        completion: Option<CharStringView<'_>>,
    ) -> Result<(), crate::ErrorKind> {
        self.print_internal(&mut std::io::stdout(), buffer, completion, false)
    }

    fn print_internal(
        &mut self,
        stdout: &mut std::io::Stdout,
        buffer: &Buffer,
        completion: Option<CharStringView<'_>>,
        transient: bool,
    ) -> Result<(), crate::ErrorKind> {
        use std::io::Write;

        clear_from(stdout, self.printed_length - self.cursor_offset)?;

        self.cursor_offset = buffer.len() - buffer.cursor();
        self.printed_length = buffer.len();

        crossterm::queue!(stdout, crossterm::style::Print(&buffer))?;

        if let Some(completion) = completion {
            use crossterm::style::Colorize;
            crossterm::queue!(
                stdout,
                crossterm::style::PrintStyledContent(crossterm::style::style(completion).blue())
            )?;
            rewind_cursor(stdout, completion.len())?;
        }

        if transient {
            Ok(())
        } else {
            rewind_cursor(stdout, self.cursor_offset)?;
            crossterm::execute!(stdout)
        }
    }

    pub(super) fn print_suggestions(
        &mut self,
        selected_index: usize,
        suggestions: &[Buffer],
    ) -> Result<(), crate::ErrorKind> {
        use std::io::Write;
        let mut stdout = std::io::stdout();

        // Print buffer
        self.print_internal(&mut stdout, &suggestions[selected_index], None, true)?;

        // Save position at the end of the buffer
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
            let length = suggestion.len();
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
