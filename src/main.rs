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
use layout::Language;
use std::io::{self, stdout, Write};
use std::time::Duration;

fn print_help() {
    println!(r#"memmon - Unified memory (RAM + SSD Swap) visualizer for Linux

Usage: memmon [OPTIONS]

Options:
  -l, --lang <LANG>  Set interface language ('en' or 'ru')
      --en           Force English interface
      --ru           Force Russian interface
  -h, --help         Print help information
  -V, --version      Print version

Controls in TUI:
  q, Q, Esc          Exit application
  l, L               Toggle language on the fly (EN <-> RU)
  Ctrl + C           Clean exit
"#);
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    for arg in &args[1..] {
        if arg == "-h" || arg == "--help" {
            print_help();
            return Ok(());
        }
        if arg == "-V" || arg == "--version" {
            println!("memmon 0.1.1");
            return Ok(());
        }
    }

    let mut lang = Language::from_env_and_args();

    // Правило 14: Первый вывод консоли начинается строго с трех переводов строк
    let init_msg = match lang {
        Language::Ru => "\n\n\nИнициализация монитора памяти memmon (Rust)...",
        Language::En => "\n\n\nInitializing memmon unified memory visualizer (Rust)...",
    };
    println!("{init_msg}");
    std::thread::sleep(Duration::from_millis(150));

    if !crossterm::terminal::is_raw_mode_enabled().unwrap_or(false) && !std::io::IsTerminal::is_terminal(&stdout()) {
        let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
        let sys = mem::get_system_memory();
        let procs = process::get_top_consumers(10);
        let frame = layout::render_frame(cols, rows, &sys, &procs, lang);
        print!("{}", frame.replace("\r\n", "\n"));
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
        let frame = layout::render_frame(cur_size.0, cur_size.1, &sys, &procs, lang);

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
                    if key.code == KeyCode::Char('l') || key.code == KeyCode::Char('L') {
                        lang = lang.toggle();
                        execute!(stdout, Clear(ClearType::All))?;
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
