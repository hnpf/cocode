use crate::agent::known_agents;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{Attribute, Color, Print, SetAttribute, SetForegroundColor},
    terminal::{self, ClearType},
};
use std::io::{self, Write};

pub fn pick_agent() -> io::Result<Option<String>> {
    let agents = known_agents();
    let mut selected: usize = 0;
    let mut stdout = io::stdout();

    terminal::enable_raw_mode()?;
    execute!(stdout, cursor::Hide)?;

    // record where the menu starts so we can return to it on redraw
    let (_, start_row) = cursor::position()?;

    draw(&agents, selected, start_row, &mut stdout)?;

    let result = loop {
        if let Event::Key(KeyEvent { code, .. }) = event::read()? {
            match code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if selected > 0 {
                        selected -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if selected < agents.len() - 1 {
                        selected += 1;
                    }
                }
                KeyCode::Enter => {
                    break Ok(Some(selected));
                }
                KeyCode::Char('q') | KeyCode::Esc => {
                    break Ok(None);
                }
                _ => {}
            }
            draw(&agents, selected, start_row, &mut stdout)?;
        }
    };

    // erase the menu before returning
    erase(start_row, menu_height(agents.len()), &mut stdout)?;

    execute!(stdout, cursor::Show)?;
    terminal::disable_raw_mode()?;

    result.map(|r| r.map(|i| agents[i].0.to_string()))
}

// total lines the menu occupies: header + blank + agents + blank + footer
fn menu_height(n: usize) -> u16 {
    (n + 4) as u16
}

fn draw(
    agents: &[(&str, &str)],
    selected: usize,
    start_row: u16,
    stdout: &mut io::Stdout,
) -> io::Result<()> {
    let (cols, _) = terminal::size()?;

    // jump back to the top of the menu every redraw
    queue!(stdout, cursor::MoveTo(0, start_row))?;

    // header
    queue!(
        stdout,
        SetAttribute(Attribute::Bold),
        SetForegroundColor(Color::Cyan),
        Print(pad("  cocode — pick an agent", cols)),
        Print("\r\n"),
        Print(pad("", cols)), // blank line
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

    // footer
    queue!(
        stdout,
        Print(pad("", cols)), // blank line
        Print("\r\n"),
        SetForegroundColor(Color::DarkGrey),
        Print(pad("  ↑/↓  j/k  enter  q", cols)),
        Print("\r\n"),
        SetAttribute(Attribute::Reset),
    )?;

    stdout.flush()
}

fn erase(start_row: u16, height: u16, stdout: &mut io::Stdout) -> io::Result<()> {
    queue!(stdout, cursor::MoveTo(0, start_row))?;
    for _ in 0..height {
        queue!(stdout, terminal::Clear(ClearType::CurrentLine), Print("\r\n"))?;
    }
    queue!(stdout, cursor::MoveTo(0, start_row))?;
    stdout.flush()
}

// pad or truncate a string to exactly `cols` wide so every row fully overwrites the previous
fn pad(s: &str, cols: u16) -> String {
    let w = cols as usize;
    let char_count = s.chars().count();
    if char_count >= w {
        s.chars().take(w).collect()
    } else {
        format!("{}{}", s, " ".repeat(w - char_count))
    }
}
