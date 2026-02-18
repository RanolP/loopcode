use std::{io, time::Duration};

use crossterm::event::{self, Event};

use crate::app::{App, InputEvent};

use super::input_map::map_input_event;

pub(crate) fn run_event_loop<H>(app: &mut App, on_input: &mut H) -> io::Result<()>
where
    H: FnMut(&mut App, InputEvent) -> bool,
{
    loop {
        match event::poll(Duration::from_millis(250)) {
            Ok(true) => {
                let Ok(raw) = event::read() else {
                    continue;
                };
                if matches!(raw, Event::FocusGained) {
                    app.set_terminal_focus(true);
                    app.render_all_windows()?;
                    continue;
                }
                if matches!(raw, Event::FocusLost) {
                    app.set_terminal_focus(false);
                    app.render_all_windows()?;
                    continue;
                }
                if matches!(raw, Event::Resize(_, _)) {
                    app.render_all_windows()?;
                    continue;
                }
                if let Some(input) = map_input_event(raw) {
                    app.note_input_activity();
                    if on_input(app, input) {
                        break;
                    }
                    app.render_all_windows()?;
                }
            }
            Ok(false) => {
                if on_input(app, InputEvent::Tick) {
                    break;
                }
                app.render_all_windows()?;
            }
            Err(_) => continue,
        }
    }

    Ok(())
}
