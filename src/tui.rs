use crate::agent::known_agents;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{self, Attribute, Color, SetAttribute, SetForegroundColor},
    terminal::{self, ClearType},
};
use std::io::{self, Write};

// returns the selected agent name, or None if user cancelled
pub fn pick_agent() -> io::Result<Option<String>> {
    let agents = known_agents();
    let mut selected: usize = 0;
    let mut stdout = io::stdout();

    terminal::enable_raw_mode()?;
    execute!(stdout, cursor::Hide)?;

    let result = run_loop(&agents, &mut selected, &mut stdout);

    execute!(stdout, cursor::Show)?;
    terminal::disable_raw_mode()?;

    result.map(|picked| picked.map(|i| agents[i].0.to_string()))
}

fn run_loop(
    agents: &[(&str, &str)],
    selected: &mut usize,
    stdout: &mut io::Stdout,
) -> io::Result<Option<usize>> {
    draw(agents, *selected, stdout)?;

    loop {
        if let Event::Key(KeyEvent { code, .. }) = event::read()? {
            match code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if *selected > 0 {
                        *selected -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if *selected < agents.len() - 1 {
                        *selected += 1;
                    }
                }
                KeyCode::Enter => {
                    clear_menu(agents.len(), stdout)?;
                    return Ok(Some(*selected));
                }
                KeyCode::Char('q') | KeyCode::Esc => {
                    clear_menu(agents.len(), stdout)?;
                    return Ok(None);
                }
                _ => {}
            }
            draw(agents, *selected, stdout)?;
        }
    }
}

fn draw(agents: &[(&str, &str)], selected: usize, stdout: &mut io::Stdout) -> io::Result<()> {
    // move to start of menu block
    queue!(
        stdout,
        cursor::MoveToColumn(0),
        terminal::Clear(ClearType::FromCursorDown)
    )?;

    // header
    queue!(
        stdout,
        SetAttribute(Attribute::Bold),
        SetForegroundColor(Color::Cyan),
        style::Print("  cocode — pick an agent\n\n"),
        SetAttribute(Attribute::Reset),
    )?;

    for (i, (name, desc)) in agents.iter().enumerate() {
        if i == selected {
            queue!(
                stdout,
                SetForegroundColor(Color::Green),
                SetAttribute(Attribute::Bold),
                style::Print(format!("  ▶  {:<10}", name)),
                SetAttribute(Attribute::Reset),
                SetForegroundColor(Color::White),
                style::Print(format!("  {desc}\n")),
                SetAttribute(Attribute::Reset),
            )?;
        } else {
            queue!(
                stdout,
                SetForegroundColor(Color::DarkGrey),
                style::Print(format!("     {:<10}", name)),
                SetAttribute(Attribute::Reset),
                style::Print(format!("  {desc}\n")),
            )?;
        }
    }

    // footer hint
    queue!(
        stdout,
        SetForegroundColor(Color::DarkGrey),
        style::Print("\n  ↑/↓ or j/k  enter  q/esc\n"),
        SetAttribute(Attribute::Reset),
    )?;

    stdout.flush()
}

fn clear_menu(n: usize, stdout: &mut io::Stdout) -> io::Result<()> {
    // n agents + header(2) + footer(2) lines
    let lines = n + 4;
    for _ in 0..lines {
        queue!(
            stdout,
            cursor::MoveUp(1),
            terminal::Clear(ClearType::CurrentLine)
        )?;
    }
    stdout.flush()
}
