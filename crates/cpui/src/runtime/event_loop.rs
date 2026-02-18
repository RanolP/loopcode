use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event};

use crate::app::{App, InputEvent};

use super::input_map::map_input_event;

pub(crate) fn run_event_loop<H>(app: &mut App, on_input: &mut H) -> io::Result<()>
where
    H: FnMut(&mut App, InputEvent) -> bool,
{
    const RESIZE_DEBOUNCE: Duration = Duration::from_millis(120);
    let mut pending_resize_at: Option<Instant> = None;

    loop {
        if flush_debounced_resize(app, &mut pending_resize_at, RESIZE_DEBOUNCE)? {
            continue;
        }

        match event::poll(Duration::from_millis(250)) {
            Ok(true) => {
                let Ok(raw) = event::read() else {
                    continue;
                };
                if matches!(raw, Event::FocusGained) {
                    app.set_terminal_focus(true);
                    if pending_resize_at.is_none() {
                        app.render_all_windows()?;
                    }
                    continue;
                }
                if matches!(raw, Event::FocusLost) {
                    app.set_terminal_focus(false);
                    if pending_resize_at.is_none() {
                        app.render_all_windows()?;
                    }
                    continue;
                }
                if matches!(raw, Event::Resize(_, _)) {
                    pending_resize_at = Some(Instant::now());
                    continue;
                }
                if let Some(input) = map_input_event(raw) {
                    if matches!(input, InputEvent::Key(_)) {
                        app.note_input_activity();
                    }
                    if on_input(app, input) {
                        break;
                    }
                    if pending_resize_at.is_none() {
                        app.render_all_windows()?;
                    }
                }
            }
            Ok(false) => {
                if on_input(app, InputEvent::Tick) {
                    break;
                }
                if pending_resize_at.is_none() {
                    app.render_all_windows()?;
                }
            }
            Err(_) => continue,
        }
    }

    Ok(())
}

fn flush_debounced_resize(
    app: &mut App,
    pending_resize_at: &mut Option<Instant>,
    debounce: Duration,
) -> io::Result<bool> {
    if let Some(at) = *pending_resize_at
        && at.elapsed() >= debounce
    {
        app.render_all_windows()?;
        *pending_resize_at = None;
        return Ok(true);
    }
    Ok(false)
}
