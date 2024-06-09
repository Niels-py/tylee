use crossterm::{
    cursor,
    event::{poll, read, Event, KeyCode},
    execute, queue,
    style::{PrintStyledContent, Stylize},
    terminal, ExecutableCommand,
};
use std::io::{self, Write};
use std::time::Duration;

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    let (mut width, mut height) = terminal::size()?;

    // create raw buffer
    terminal::enable_raw_mode()?;
    execute!(io::stdout(), terminal::EnterAlternateScreen)?;
    let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.".to_string();
    let mut lines = split_into_lines(&text, width as usize / 2);

    init_lines(&lines, width, height)?;
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

                    lines = split_into_lines(&text, (width / 2) as usize);

                    init_lines(&lines, width, height)?;
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
                            queue!(io::stdout(), PrintStyledContent(key.green()))?;
                        } else if key == ' ' {
                            queue!(io::stdout(), PrintStyledContent("█".red()))?;
                        } else {
                            queue!(io::stdout(), PrintStyledContent(key.red()))?;
                        }

                        if cursor_index == lines[cursor_line_index].len() - 1 {
                            cursor_line_index += 1;
                            cursor_index = 0;
                            if cursor_line_index == lines.len() {
                                break;
                            }
                            queue!(
                                io::stdout(),
                                cursor::MoveTo(
                                    (width - lines[cursor_line_index].len() as u16) / 2,
                                    (height - lines.len() as u16) / 2 + cursor_line_index as u16,
                                )
                            )?;
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

                            queue!(
                                io::stdout(),
                                cursor::MoveTo(
                                    (width - lines[cursor_line_index].len() as u16) / 2
                                        + lines[cursor_line_index].len() as u16
                                        - 1,
                                    (height - lines.len() as u16) / 2 + cursor_line_index as u16,
                                )
                            )?;
                            queue!(
                                io::stdout(),
                                PrintStyledContent(
                                    lines[cursor_line_index].chars().last().unwrap().blue(),
                                )
                            )?;
                            queue!(io::stdout(), cursor::MoveLeft(1))?;
                        } else {
                            cursor_index -= 1;
                            queue!(io::stdout(), cursor::MoveLeft(1))?;
                            queue!(
                                io::stdout(),
                                PrintStyledContent(
                                    lines[cursor_line_index]
                                        .chars()
                                        .nth(cursor_index)
                                        .unwrap()
                                        .blue(),
                                )
                            )?;
                            queue!(io::stdout(), cursor::MoveLeft(1))?;
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

    // disable raw buffer
    terminal::disable_raw_mode()?;
    execute!(io::stdout(), terminal::LeaveAlternateScreen)?;

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

fn init_lines(lines: &[String], width: u16, height: u16) -> io::Result<()> {
    queue!(io::stdout(), terminal::Clear(terminal::ClearType::All))?;
    let start_height = (height - lines.len() as u16) / 2;
    for (i, line) in lines.iter().enumerate() {
        let start_width = (width - line.len() as u16) / 2;
        queue!(
            io::stdout(),
            cursor::MoveTo(start_width, start_height + i as u16)
        )?;
        queue!(io::stdout(), PrintStyledContent(line.clone().blue()))?;
    }

    queue!(
        io::stdout(),
        cursor::MoveTo((width - lines[0].len() as u16) / 2, start_height,)
    )?;
    Ok(())
}
