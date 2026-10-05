mod layout;
mod mem;
mod process;
mod terminal;

use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{Clear, ClearType},
};
use std::io::{self, stdout, Write};
use std::time::Duration;

fn main() -> io::Result<()> {
    // Правило 14: Первый вывод консоли начинается строго с трех переводов строк
    println!("\n\n\nИнициализация монитора памяти memmon (Rust)...");
    std::thread::sleep(Duration::from_millis(150));

    if !crossterm::terminal::is_raw_mode_enabled().unwrap_or(false) && !std::io::IsTerminal::is_terminal(&stdout()) {
        let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
        let sys = mem::get_system_memory();
        let procs = process::get_top_consumers(10);
        println!("{}", layout::render_frame(cols, rows, &sys, &procs));
        return Ok(());
    }

    let _guard = terminal::TerminalGuard::init()?;
    let mut stdout = stdout();

    let mut last_size = (0, 0);

    loop {
        let cur_size = crossterm::terminal::size().unwrap_or((80, 24));
        if cur_size != last_size {
            last_size = cur_size;
            execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
        } else {
            execute!(stdout, MoveTo(0, 0))?;
        }

        let sys = mem::get_system_memory();
        let procs = process::get_top_consumers(10);
        let frame = layout::render_frame(cur_size.0, cur_size.1, &sys, &procs);

        write!(stdout, "{frame}")?;
        stdout.flush()?;

        if event::poll(Duration::from_millis(400))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.code == KeyCode::Char('q')
                        || key.code == KeyCode::Char('Q')
                        || key.code == KeyCode::Esc
                        || (key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c'))
                    {
                        break;
                    }
                }
                Event::Resize(_, _) => {
                    execute!(stdout, Clear(ClearType::All))?;
                }
                _ => {}
            }
        }
    }

    Ok(())
}
