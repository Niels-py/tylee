use crossterm::{
    cursor,
    event::{poll, read, Event, KeyCode},
    execute, queue,
    style::{Color, Print, PrintStyledContent, Stylize},
    terminal,
};
use rand::seq::IteratorRandom;
use rand::thread_rng;
use std::io::{self, IsTerminal, Read, Write};
use std::time::{Duration, SystemTime};

mod word_lists;

fn main() -> io::Result<()> {
    let (mut width, mut height) = terminal::size()?;

    // stdin
    let mut text: String = String::new();
    if !io::stdin().is_terminal() {
        // so, if stdin is from a program piped into this program
        io::stdin().read_to_string(&mut text)?;
    }
    text = text.trim().to_string();

    // create raw buffer
    terminal::enable_raw_mode()?;
    execute!(io::stdout(), terminal::EnterAlternateScreen)?;
    if text.is_empty() {
        text = get_text(50)?;
    }
    let time: u64 = 30;
    let mut lines = split_into_lines(&text, width as usize / 2);

    init_lines(&lines, width, height)?;
    io::stdout().flush()?;

    let mut cursor_index = 0;
    let mut cursor_line_index = 0;

    let start_time = SystemTime::now();
    let mut remaining_time = time;

    while remaining_time != 0 {
        draw_timer(&mut remaining_time, start_time, width, time)?;
        if poll(Duration::from_millis(500))? {
            match read()? {
                Event::Resize(w, h) => {
                    width = w;
                    height = h;
                    cursor_index = 0;
                    cursor_line_index = 0;

                    lines = split_into_lines(&text, (width / 2) as usize);

                    init_lines(&lines, width, height)?;
                    io::stdout().flush()?;
                }
                Event::FocusGained => {
                    execute!(io::stdout(), cursor::EnableBlinking)?;
                }
                Event::FocusLost => {
                    execute!(io::stdout(), cursor::DisableBlinking)?;
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

                        io::stdout().flush()?;
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

                        io::stdout().flush()?;
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

    let time_typed = start_time.elapsed().unwrap().as_secs();

    let mut words_typed = 0;
    let mut chars_typed = 0;
    for (index, line) in lines.iter().enumerate() {
        words_typed += line.split_whitespace().count();
        chars_typed += line.len();
        if index > cursor_line_index {
            words_typed += line[0..cursor_index].split_whitespace().count();
            chars_typed += cursor_index;
            break;
        }
    }
    let pure_wpm = words_typed as f64 * (60. / time_typed as f64);
    let raw_wpm = chars_typed as f64 / 5. * (60. / time_typed as f64);

    println!(" time typed: {}", time_typed);
    println!("words typed: {}", words_typed);
    println!("chars typed: {}", chars_typed);
    println!("   pure wpm: {:.2}", pure_wpm);
    println!("    raw wpm: {:.2}", raw_wpm);

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
        cursor::MoveTo((width - lines[0].len() as u16) / 2, start_height)
    )?;
    Ok(())
}

fn draw_timer(
    remaining_time: &mut u64,
    start_time: SystemTime,
    width: u16,
    time: u64,
) -> io::Result<()> {
    *remaining_time = time - start_time.elapsed().unwrap().as_secs();

    // draw number
    execute!(
        io::stdout(),
        cursor::SavePosition,
        cursor::MoveTo(1, 1),
        // print extra whitespaces so there aren't any trailing digits
        PrintStyledContent((remaining_time.to_string() + "     ").with(Color::Yellow)),
        cursor::RestorePosition
    )?;

    // draw bar
    execute!(
        io::stdout(),
        cursor::SavePosition,
        cursor::MoveTo(0, 0),
        Print(" ".repeat(width as usize)),
        cursor::MoveTo(0, 0),
        PrintStyledContent(
            "#".repeat((width as f64 * (*remaining_time as f64 / time as f64)) as usize)
                .with(Color::Green)
        ),
        cursor::RestorePosition
    )?;

    Ok(())
}

fn get_text(amount_of_words: usize) -> io::Result<String> {
    let mut text = String::new();
    let mut rng = thread_rng();
    for _ in 0..amount_of_words {
        text.push_str(word_lists::DEFAULT_ENGLISH.iter().choose(&mut rng).unwrap());
        text.push(' ');
    }
    Ok(text)
}
