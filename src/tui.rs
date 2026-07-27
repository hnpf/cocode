use crate::agent::known_agents;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::{Attribute, Color, Print, SetAttribute, SetForegroundColor},
    terminal::{self, ClearType},
};
use std::io::{self, Write};

pub fn pick_agent(last_used: Option<usize>) -> io::Result<Option<String>> {
    let agents = known_agents();
    let mut selected: usize = last_used.unwrap_or(0).min(agents.len() - 1);
    let mut stdout = io::stdout();

    terminal::enable_raw_mode()?;
    execute!(stdout, cursor::Hide)?;

    // restore terminal on panic so the shell isn't left in raw mode
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = execute!(io::stdout(), cursor::Show);
        let _ = terminal::disable_raw_mode();
        default_hook(info);
    }));

    let (_, start_row) = cursor::position()?;
    draw(&agents, selected, start_row, &mut stdout)?;

    let result = loop {
        if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
            match (code, modifiers) {
                // ctrl+c / ctrl+d cancel cleanly
                (KeyCode::Char('c'), KeyModifiers::CONTROL)
                | (KeyCode::Char('d'), KeyModifiers::CONTROL) => break Ok(None),

                (KeyCode::Up, _) | (KeyCode::Char('k'), _) => {
                    selected = selected.saturating_sub(1);
                }
                (KeyCode::Down, _) | (KeyCode::Char('j'), _) => {
                    if selected < agents.len() - 1 {
                        selected += 1;
                    }
                }
                (KeyCode::Enter, _) => break Ok(Some(selected)),
                (KeyCode::Char('q'), _) | (KeyCode::Esc, _) => break Ok(None),
                _ => continue, // ignore unhandled keys — no redraw
            }
            draw(&agents, selected, start_row, &mut stdout)?;
        }
    };

    erase(start_row, menu_height(agents.len()), &mut stdout)?;
    execute!(stdout, cursor::Show)?;
    terminal::disable_raw_mode()?;

    // restore default panic hook
    let _ = std::panic::take_hook();

    result.map(|r| r.map(|i| agents[i].0.to_string()))
}

fn menu_height(n: usize) -> u16 {
    (n + 13) as u16
}

fn draw(
    agents: &[(&str, &str)],
    selected: usize,
    start_row: u16,
    stdout: &mut io::Stdout,
) -> io::Result<()> {
    let (cols, _) = terminal::size()?;

    queue!(stdout, cursor::MoveTo(0, start_row))?;

    queue!(
        stdout,
        Print(pad("", cols)),
        Print("\r\n"),
        Print(pad("", cols)),
        Print("\r\n"),
        SetAttribute(Attribute::Bold),
    )?;
    print_wordmark(stdout, cols)?;
    queue!(
        stdout,
        SetForegroundColor(Color::DarkGrey),
        Print(pad("  pick an agent", cols)),
        Print("\r\n"),
        Print(pad("", cols)),
        Print("\r\n"),
        SetAttribute(Attribute::Reset),
    )?;

    for (i, (name, desc)) in agents.iter().enumerate() {
        let line = if i == selected {
            format!("  ▶  {:<10}  {}", name, desc)
        } else {
            format!("     {:<10}  {}", name, desc)
        };
        let line = pad(&line, cols);

        if i == selected {
            queue!(
                stdout,
                SetForegroundColor(Color::Green),
                SetAttribute(Attribute::Bold),
                Print(line),
                SetAttribute(Attribute::Reset),
                Print("\r\n"),
            )?;
        } else {
            queue!(
                stdout,
                SetForegroundColor(Color::DarkGrey),
                Print(line),
                SetAttribute(Attribute::Reset),
                Print("\r\n"),
            )?;
        }
    }

    queue!(
        stdout,
        Print(pad("", cols)),
        Print("\r\n"),
        SetForegroundColor(Color::DarkGrey),
        Print(pad("  ↑/↓  j/k  enter  q/ctrl+c", cols)),
        Print("\r\n"),
        Print(pad("", cols)),
        Print("\r\n"),
        Print(pad("", cols)),
        Print("\r\n"),
        SetAttribute(Attribute::Reset),
    )?;

    stdout.flush()
}

/// Print the cocode wordmark as a blue-to-pink truecolor ASCII banner.
fn print_wordmark(stdout: &mut io::Stdout, cols: u16) -> io::Result<()> {
    const WORDMARK: [&str; 5] = [
        " ██████  ██████   ██████  ██████  ██████  ███████",
        "██      ██    ██ ██      ██    ██ ██    ██ ██     ",
        "██      ██    ██ ██      ██    ██ ██    ██ █████  ",
        "██      ██    ██ ██      ██    ██ ██    ██ ██     ",
        " ██████  ██████   ██████  ██████  ██████  ███████",
    ];
    let width = cols.saturating_sub(2) as usize;

    for line in WORDMARK {
        queue!(stdout, Print("  "))?;
        for (column, character) in line.chars().take(width).enumerate() {
            if character == ' ' {
                queue!(stdout, Print(character))?;
            } else {
                queue!(
                    stdout,
                    SetForegroundColor(gradient_color(column, line.chars().count())),
                    Print(character)
                )?;
            }
        }
        queue!(stdout, Print(" ".repeat(width.saturating_sub(line.chars().count()))), Print("\r\n"))?;
    }

    Ok(())
}

fn gradient_color(position: usize, width: usize) -> Color {
    let t = position as f32 / width.saturating_sub(1).max(1) as f32;
    let start = (83.0, 169.0, 255.0);
    let end = (255.0, 91.0, 181.0);

    Color::Rgb {
        r: (start.0 + (end.0 - start.0) * t) as u8,
        g: (start.1 + (end.1 - start.1) * t) as u8,
        b: (start.2 + (end.2 - start.2) * t) as u8,
    }
}

fn erase(start_row: u16, height: u16, stdout: &mut io::Stdout) -> io::Result<()> {
    queue!(stdout, cursor::MoveTo(0, start_row))?;
    for _ in 0..height {
        queue!(stdout, terminal::Clear(ClearType::CurrentLine), Print("\r\n"))?;
    }
    queue!(stdout, cursor::MoveTo(0, start_row))?;
    stdout.flush()
}

fn pad(s: &str, cols: u16) -> String {
    let w = cols as usize;
    let n = s.chars().count();
    if n >= w {
        s.chars().take(w).collect()
    } else {
        format!("{}{}", s, " ".repeat(w - n))
    }
}
