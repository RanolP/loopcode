use super::{FocusEntry, FocusKind, FocusNavOutcome, FocusState, UiInputEvent, UiKeyInput};

impl FocusState {
    pub fn handle_navigation(
        &mut self,
        event: UiInputEvent,
        entries: &[FocusEntry],
    ) -> FocusNavOutcome {
        self.expire_quit_arm();

        let key = match event {
            UiInputEvent::Key(key) => key,
            UiInputEvent::Tick => return FocusNavOutcome::Ignored,
            UiInputEvent::ScrollLines(_) => {
                self.disarm_quit();
                return FocusNavOutcome::Ignored;
            }
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
                if self.quit_armed() {
                    self.disarm_quit();
                    FocusNavOutcome::RequestQuit
                } else {
                    self.arm_quit();
                    FocusNavOutcome::Handled
                }
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
            self.disarm_quit();
        }
        out
    }
}
