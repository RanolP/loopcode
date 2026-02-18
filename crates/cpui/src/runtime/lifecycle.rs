use std::{
    io::{self, Write},
    sync::atomic::{AtomicBool, Ordering},
};

use crossterm::event::{
    DisableMouseCapture, EnableMouseCapture, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags,
    PushKeyboardEnhancementFlags,
};
use crossterm::execute;
use crossterm::style::ResetColor;
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor, terminal::Clear, terminal::ClearType};

static ALT_SCREEN_ACTIVE: AtomicBool = AtomicBool::new(false);
const KEYBOARD_FLAGS: KeyboardEnhancementFlags =
    KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
        .union(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
        .union(KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS)
        .union(KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES);

pub(crate) fn is_alt_screen_active() -> bool {
    ALT_SCREEN_ACTIVE.load(Ordering::Relaxed)
}

pub(crate) fn enter_terminal() -> io::Result<TerminalGuard> {
    // Runtime contract:
    // 1) enable raw mode + enter alternate screen
    // 2) mark alt-screen active
    // 3) return guard that restores terminal and marks inactive on drop
    terminal::enable_raw_mode()?;
    if let Err(err) = execute!(
        io::stdout(),
        EnterAlternateScreen,
        Clear(ClearType::All),
        cursor::MoveTo(0, 0),
        EnableMouseCapture
    ) {
        let _ = terminal::disable_raw_mode();
        return Err(err);
    }
    ALT_SCREEN_ACTIVE.store(true, Ordering::Relaxed);
    let _ = execute!(io::stdout(), PushKeyboardEnhancementFlags(KEYBOARD_FLAGS));

    Ok(TerminalGuard)
}

pub(crate) struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let mut out = io::stdout();
        let _ = terminal::disable_raw_mode();
        let _ = execute!(
            out,
            DisableMouseCapture,
            PopKeyboardEnhancementFlags,
            ResetColor,
            cursor::Show
        );
        let _ = execute!(out, LeaveAlternateScreen);
        ALT_SCREEN_ACTIVE.store(false, Ordering::Relaxed);
        let _ = out.flush();
    }
}
