use crossterm::{
    cursor,
    event::{poll, read, Event, KeyCode},
    style::{PrintStyledContent, Stylize},
    terminal, ExecutableCommand, QueueableCommand,
};
use std::io::{self, Write};
use std::time::Duration;

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    let (mut width, mut height) = terminal::size()?;

    terminal::enable_raw_mode()?;

    stdout.execute(terminal::Clear(terminal::ClearType::All))?;

    let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";

    let mut lines = split_into_lines(text, (width / 2) as usize);

    init_lines(&mut stdout, &lines, width, height)?;
    stdout.flush()?;

    let mut cursor_index = 0;
    let mut cursor_line_index = 0;

    loop {
        if poll(Duration::from_millis(500))? {
            match read()? {
                Event::Resize(w, h) => {
                    width = w;
                    height = h;
                    cursor_index = 0;
                    cursor_line_index = 0;

                    lines = split_into_lines(text, (width / 2) as usize);

                    init_lines(&mut stdout, &lines, width, height)?;
                    stdout.flush()?;
                }
                Event::FocusGained => {
                    stdout.execute(cursor::EnableBlinking)?;
                }
                Event::FocusLost => {
                    stdout.execute(cursor::DisableBlinking)?;
                }
                // Mouse capture disabled by default
                // Event::Mouse(_) => {}
                Event::Paste(_) => {}

                Event::Key(keyevent) => match keyevent.code {
                    KeyCode::Char(key) => {
                        if key == lines[cursor_line_index].chars().nth(cursor_index).unwrap() {
                            stdout.queue(PrintStyledContent(key.green()))?;
                        } else if key == ' ' {
                            stdout.queue(PrintStyledContent("█".red()))?;
                        } else {
                            stdout.queue(PrintStyledContent(key.red()))?;
                        }

                        if cursor_index == lines[cursor_line_index].len() - 1 {
                            cursor_line_index += 1;
                            cursor_index = 0;
                            if cursor_line_index == lines.len() {
                                break;
                            }
                            stdout.queue(cursor::MoveTo(
                                width / 2 - (lines[cursor_line_index].len() / 2) as u16,
                                ((height / 2) as usize - lines.len() / 2 + cursor_line_index)
                                    as u16,
                            ))?;
                        } else {
                            cursor_index += 1;
                        }

                        stdout.flush()?;
                    }
                    KeyCode::Backspace => {
                        if cursor_index == 0 && cursor_line_index == 0 {
                            continue;
                        } else if cursor_index == 0 {
                            cursor_line_index -= 1;
                            cursor_index = lines[cursor_line_index].len() - 1;

                            stdout
                                .queue(cursor::MoveTo(
                                    width / 2 - (lines[cursor_line_index].len() / 2) as u16
                                        + (lines[cursor_line_index].len() - 1) as u16,
                                    ((height / 2) as usize - lines.len() / 2 + cursor_line_index)
                                        as u16,
                                ))?
                                .queue(PrintStyledContent(
                                    lines[cursor_line_index].chars().last().unwrap().blue(),
                                ))?
                                .queue(cursor::MoveLeft(1))?;
                        } else {
                            cursor_index -= 1;
                            stdout
                                .queue(cursor::MoveLeft(1))?
                                .queue(PrintStyledContent(
                                    lines[cursor_line_index]
                                        .chars()
                                        .nth(cursor_index)
                                        .unwrap()
                                        .blue(),
                                ))?
                                .queue(cursor::MoveLeft(1))?;
                        }

                        stdout.flush()?;
                    }
                    KeyCode::Esc => break,
                    _ => {}
                },
                _ => break,
            }
        }
    }

    // clear terminal before exiting
    terminal::disable_raw_mode()?;
    stdout
        .queue(terminal::Clear(terminal::ClearType::All))?
        .queue(cursor::MoveTo(0, 0))?;
    stdout.flush()?;

    Ok(())
}

fn split_into_lines(text: &str, length_of_line: usize) -> Vec<String> {
    let words = text.split_whitespace();

    let mut lines: Vec<String> = Vec::new();
    let mut line: String = String::new();

    for word in words {
        if line.len() + word.len() > length_of_line {
            lines.push(line);
            line = String::new();
        }
        line.push_str(word);
        line.push(' ');
    }

    // pop of space at the end
    line.pop();
    lines.push(line);

    lines
}

fn init_lines(
    stdout: &mut io::Stdout,
    lines: &[String],
    width: u16,
    height: u16,
) -> io::Result<()> {
    stdout.queue(terminal::Clear(terminal::ClearType::All))?;
    let start_height = height / 2 - lines.len() as u16 / 2;
    for (i, line) in lines.iter().enumerate() {
        let start_width = width / 2 - (line.len() / 2) as u16;
        stdout
            .queue(cursor::MoveTo(start_width, start_height + i as u16))?
            .queue(PrintStyledContent(line.clone().blue()))?;
    }

    stdout.queue(cursor::MoveTo(
        width / 2 - (lines[0].len() / 2) as u16,
        start_height,
    ))?;
    Ok(())
}
