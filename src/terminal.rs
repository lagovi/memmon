use crossterm::{
    cursor::{Hide, Show},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use std::io::{self, stdout, Write};
use std::sync::atomic::{AtomicBool, Ordering};

static CLEANUP_DONE: AtomicBool = AtomicBool::new(false);

pub struct TerminalGuard;

impl TerminalGuard {
    pub fn init() -> io::Result<Self> {
        let default_panic = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            Self::restore();
            default_panic(info);
        }));

        enable_raw_mode()?;
        let mut out = stdout();
        execute!(out, EnterAlternateScreen, Hide)?;
        out.flush()?;
        Ok(TerminalGuard)
    }

    pub fn restore() {
        if !CLEANUP_DONE.swap(true, Ordering::SeqCst) {
            let mut out = stdout();
            let _ = execute!(out, Show, LeaveAlternateScreen);
            let _ = disable_raw_mode();
            let _ = out.flush();
        }
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        Self::restore();
    }
}
