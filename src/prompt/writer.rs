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
        use std::io::Write;
        let mut stdout = std::io::stdout();

        rewind_cursor(&mut stdout, self.printed_length - self.cursor_offset)?;

        crossterm::queue!(
            stdout,
            crossterm::style::ResetColor,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::FromCursorDown),
        )?;

        self.cursor_offset = buffer.len() - buffer.cursor();
        self.printed_length = buffer.len();

        crossterm::queue!(stdout, crossterm::style::Print(&buffer),)?;

        if let Some(completion) = completion {
            crossterm::queue!(
                stdout,
                crossterm::style::SetForegroundColor(crossterm::style::Color::Blue),
                crossterm::style::Print(completion),
                crossterm::style::ResetColor,
            )?;
            rewind_cursor(&mut stdout, completion.len())?;
        }

        rewind_cursor(&mut stdout, self.cursor_offset)?;

        crossterm::execute!(stdout)
    }
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

impl std::ops::Drop for Writer {
    // Allowed because this is a drop and the previous construction already managed the get through
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        use std::io::Write;
        crossterm::terminal::disable_raw_mode();

        let mut stdout = std::io::stdout();

        if let Some(prompt_length) = self.erase_on_drop {
            rewind_cursor(&mut stdout, self.printed_length + prompt_length);
            crossterm::queue!(
                stdout,
                crossterm::terminal::Clear(crossterm::terminal::ClearType::FromCursorDown)
            );
        } else {
            fast_forward_cursor(&mut stdout, self.cursor_offset);
            crossterm::queue!(
                stdout,
                crossterm::terminal::Clear(crossterm::terminal::ClearType::FromCursorDown),
                crossterm::style::Print('\n')
            );
        }
        crossterm::execute!(stdout, crossterm::style::ResetColor);
    }
}
