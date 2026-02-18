use super::{FocusEntry, FocusKind, FocusNavOutcome, FocusState, UiInputEvent, UiKeyInput};

impl FocusState {
    pub fn handle_navigation(
        &mut self,
        event: UiInputEvent,
        entries: &[FocusEntry],
    ) -> FocusNavOutcome {
        let UiInputEvent::Key(key) = event else {
            self.quit_armed = false;
            return FocusNavOutcome::Ignored;
        };

        let focused_kind = self.focused_entry(entries).map(|entry| entry.kind);
        let out = match key {
            UiKeyInput::Esc => {
                let moved_parent = self.focus_parent(entries);
                if moved_parent {
                    FocusNavOutcome::Handled
                } else {
                    FocusNavOutcome::Ignored
                }
            }
            UiKeyInput::Interrupt => {
                if self.quit_armed {
                    self.quit_armed = false;
                    FocusNavOutcome::RequestQuit
                } else {
                    self.quit_armed = true;
                    FocusNavOutcome::Handled
                }
            }
            UiKeyInput::Tab => {
                self.focus_next(entries);
                FocusNavOutcome::Handled
            }
            UiKeyInput::BackTab => {
                self.focus_prev(entries);
                FocusNavOutcome::Handled
            }
            UiKeyInput::Enter if focused_kind != Some(FocusKind::TextInput) => {
                if self.focus_first_child(entries) {
                    FocusNavOutcome::Handled
                } else {
                    FocusNavOutcome::Ignored
                }
            }
            UiKeyInput::Left | UiKeyInput::Up if focused_kind != Some(FocusKind::TextInput) => {
                if self.focus_prev_sibling(entries) || self.focus_prev_peer_branch(entries) {
                    FocusNavOutcome::Handled
                } else {
                    FocusNavOutcome::Ignored
                }
            }
            UiKeyInput::Right | UiKeyInput::Down
                if focused_kind != Some(FocusKind::TextInput) =>
            {
                if self.focus_next_sibling(entries) || self.focus_next_peer_branch(entries) {
                    FocusNavOutcome::Handled
                } else {
                    FocusNavOutcome::Ignored
                }
            }
            _ => FocusNavOutcome::Ignored,
        };

        if key != UiKeyInput::Interrupt {
            self.quit_armed = false;
        }
        out
    }
}
