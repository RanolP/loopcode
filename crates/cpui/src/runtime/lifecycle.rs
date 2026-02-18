use std::{
    io::{self, Write},
    sync::atomic::{AtomicBool, Ordering},
};

use crossterm::event::{
    DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture,
    KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::execute;
use crossterm::style::ResetColor;
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor, terminal::Clear, terminal::ClearType};

static ALT_SCREEN_ACTIVE: AtomicBool = AtomicBool::new(false);
// NOTE: crossterm currently does not expose cursor-shape APIs (DECSCUSR),
// so we emit raw CSI sequences for blinking block cursor and reset.
const CURSOR_COLOR_OSC: &str = "\x1b]12;#a277ff\x07";
const RESET_CURSOR_COLOR_OSC: &str = "\x1b]112\x07";
const BLOCK_CURSOR_CSI: &str = "\x1b[2 q";
const RESET_CURSOR_STYLE_CSI: &str = "\x1b[0 q";
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
        EnableMouseCapture,
        EnableFocusChange
    ) {
        let _ = terminal::disable_raw_mode();
        return Err(err);
    }
    ALT_SCREEN_ACTIVE.store(true, Ordering::Relaxed);
    let _ = execute!(io::stdout(), PushKeyboardEnhancementFlags(KEYBOARD_FLAGS));
    let _ = io::stdout().write_all(CURSOR_COLOR_OSC.as_bytes());
    let _ = io::stdout().write_all(BLOCK_CURSOR_CSI.as_bytes());
    let _ = io::stdout().flush();

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
            DisableFocusChange,
            PopKeyboardEnhancementFlags,
            ResetColor,
            cursor::Show
        );
        let _ = out.write_all(RESET_CURSOR_COLOR_OSC.as_bytes());
        let _ = out.write_all(RESET_CURSOR_STYLE_CSI.as_bytes());
        let _ = execute!(out, LeaveAlternateScreen);
        ALT_SCREEN_ACTIVE.store(false, Ordering::Relaxed);
        let _ = out.flush();
    }
}
