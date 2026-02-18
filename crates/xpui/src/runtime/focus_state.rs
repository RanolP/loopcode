use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::FocusId;

use super::{FocusEntry, FocusKind, FocusPath};

#[derive(Clone, Debug, Default)]
pub struct FocusState {
    focused: Option<FocusId>,
    focused_path: Option<FocusPath>,
    last_child_by_parent: HashMap<FocusPath, FocusPath>,
    pub(crate) quit_armed: bool,
    pub(crate) quit_armed_at: Option<Instant>,
}

impl FocusState {
    pub fn focused(&self) -> Option<FocusId> {
        self.focused
    }

    pub fn focused_path(&self) -> Option<&FocusPath> {
        self.focused_path.as_ref()
    }

    pub fn quit_armed(&self) -> bool {
        self.quit_armed
            && self
                .quit_armed_at
                .map(|at| at.elapsed() < Duration::from_secs(2))
                .unwrap_or(false)
    }

    pub fn expire_quit_arm(&mut self) {
        if self.quit_armed
            && self
                .quit_armed_at
                .map(|at| at.elapsed() >= Duration::from_secs(2))
                .unwrap_or(true)
        {
            self.quit_armed = false;
            self.quit_armed_at = None;
        }
    }

    pub(crate) fn arm_quit(&mut self) {
        self.quit_armed = true;
        self.quit_armed_at = Some(Instant::now());
    }

    pub(crate) fn disarm_quit(&mut self) {
        self.quit_armed = false;
        self.quit_armed_at = None;
    }

    pub fn is_focused(&self, id: FocusId) -> bool {
        self.focused == Some(id)
    }

    pub fn set_focused(&mut self, id: FocusId) {
        self.focused = Some(id);
        self.focused_path = None;
    }

    pub fn set_focused_entry(&mut self, entry: &FocusEntry) {
        self.focused = Some(entry.id);
        self.focused_path = Some(entry.path.clone());
    }

    pub fn clear_focus(&mut self) {
        self.focused = None;
        self.focused_path = None;
        self.last_child_by_parent.clear();
        self.disarm_quit();
    }

    pub fn ensure_valid(&mut self, entries: &[FocusEntry]) {
        if entries.is_empty() {
            self.focused = None;
            self.focused_path = None;
            return;
        }

        if let Some(path) = &self.focused_path
            && let Some(entry) = entries.iter().find(|entry| &entry.path == path)
        {
            self.focused = Some(entry.id);
            return;
        }

        if let Some(id) = self.focused
            && let Some(entry) = entries.iter().find(|entry| entry.id == id)
        {
            self.focused = Some(entry.id);
            self.focused_path = Some(entry.path.clone());
            return;
        }

        self.set_focused_entry(&entries[0]);
    }

    pub fn focus_next(&mut self, entries: &[FocusEntry]) {
        if entries.is_empty() {
            self.clear_focus();
            return;
        }

        let idx = self.current_index(entries).unwrap_or(0).saturating_add(1) % entries.len();
        self.set_focused_entry(&entries[idx]);
    }

    pub fn focus_prev(&mut self, entries: &[FocusEntry]) {
        if entries.is_empty() {
            self.clear_focus();
            return;
        }
        let idx = match self.current_index(entries) {
            Some(0) | None => entries.len() - 1,
            Some(i) => i - 1,
        };
        self.set_focused_entry(&entries[idx]);
    }

    pub fn focus_next_sibling(&mut self, entries: &[FocusEntry]) -> bool {
        self.focus_sibling(entries, true)
    }

    pub fn focus_prev_sibling(&mut self, entries: &[FocusEntry]) -> bool {
        self.focus_sibling(entries, false)
    }

    pub fn focus_next_peer_branch(&mut self, entries: &[FocusEntry]) -> bool {
        self.focus_peer_branch(entries, true)
    }

    pub fn focus_prev_peer_branch(&mut self, entries: &[FocusEntry]) -> bool {
        self.focus_peer_branch(entries, false)
    }

    pub fn focus_parent(&mut self, entries: &[FocusEntry]) -> bool {
        let Some(path) = self.focused_path.clone() else {
            return false;
        };
        for depth in (1..path.0.len()).rev() {
            let ancestor = FocusPath(path.0[..depth].to_vec());
            if let Some(entry) = entries.iter().find(|entry| entry.path == ancestor) {
                if matches!(entry.kind, FocusKind::ScrollRegion) {
                    self.last_child_by_parent.insert(ancestor, path.clone());
                }
                self.set_focused_entry(entry);
                return true;
            }
        }
        false
    }

    pub fn focused_entry<'a>(&self, entries: &'a [FocusEntry]) -> Option<&'a FocusEntry> {
        self.current_index(entries).map(|idx| &entries[idx])
    }

    pub fn focus_first_child(&mut self, entries: &[FocusEntry]) -> bool {
        let Some(current_idx) = self.current_index(entries) else {
            return false;
        };
        let current = &entries[current_idx];
        if matches!(current.kind, FocusKind::ScrollRegion)
            && let Some(saved_child) = self.last_child_by_parent.get(&current.path)
            && let Some(entry) = entries.iter().find(|entry| entry.path == *saved_child)
        {
            self.set_focused_entry(entry);
            return true;
        }
        let mut candidates = entries
            .iter()
            .filter(|entry| {
                entry.path.0.len() > current.path.0.len()
                    && entry.path.0.get(..current.path.0.len()) == Some(current.path.0.as_slice())
            })
            .collect::<Vec<_>>();
        if candidates.is_empty() {
            return false;
        }
        candidates.sort_by(|a, b| {
            a.path
                .0
                .len()
                .cmp(&b.path.0.len())
                .then_with(|| a.path.0.cmp(&b.path.0))
        });
        self.set_focused_entry(candidates[0]);
        true
    }

    pub fn remember_child(
        &mut self,
        parent_id: FocusId,
        child_id: FocusId,
        entries: &[FocusEntry],
    ) -> bool {
        let Some(parent) = entries.iter().find(|entry| entry.id == parent_id) else {
            return false;
        };
        let Some(child) = entries.iter().find(|entry| entry.id == child_id) else {
            return false;
        };
        if child.path.0.len() <= parent.path.0.len()
            || child.path.0.get(..parent.path.0.len()) != Some(parent.path.0.as_slice())
        {
            return false;
        }
        self.last_child_by_parent
            .insert(parent.path.clone(), child.path.clone());
        true
    }

    pub(crate) fn current_index(&self, entries: &[FocusEntry]) -> Option<usize> {
        if let Some(path) = &self.focused_path
            && let Some(idx) = entries.iter().position(|entry| &entry.path == path)
        {
            return Some(idx);
        }
        self.focused
            .and_then(|id| entries.iter().position(|entry| entry.id == id))
    }

    fn focus_sibling(&mut self, entries: &[FocusEntry], next: bool) -> bool {
        let Some(current_idx) = self.current_index(entries) else {
            return false;
        };
        let current = &entries[current_idx];
        let mut siblings = entries
            .iter()
            .filter(|entry| {
                entry.path.0.len() == current.path.0.len()
                    && entry.path.0.get(..entry.path.0.len().saturating_sub(1))
                        == current.path.0.get(..current.path.0.len().saturating_sub(1))
            })
            .collect::<Vec<_>>();
        if siblings.len() <= 1 {
            return false;
        }
        siblings.sort_by_key(|entry| &entry.path.0);
        let Some(pos) = siblings.iter().position(|entry| entry.id == current.id) else {
            return false;
        };
        let target = if next {
            siblings[(pos + 1) % siblings.len()]
        } else {
            siblings[(pos + siblings.len() - 1) % siblings.len()]
        };
        self.set_focused_entry(target);
        true
    }

    fn focus_peer_branch(&mut self, entries: &[FocusEntry], next: bool) -> bool {
        let Some(current_idx) = self.current_index(entries) else {
            return false;
        };
        let current = &entries[current_idx];
        let path = &current.path.0;

        for level in (0..path.len()).rev() {
            let parent = &path[..level];
            let current_slot = path[level];

            let mut sibling_slots = entries
                .iter()
                .filter_map(|entry| {
                    if entry.path.0.len() > level && entry.path.0.get(..level) == Some(parent) {
                        Some(entry.path.0[level])
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            sibling_slots.sort_unstable();
            sibling_slots.dedup();

            let target_slot = if next {
                sibling_slots.into_iter().find(|slot| *slot > current_slot)
            } else {
                sibling_slots
                    .into_iter()
                    .rev()
                    .find(|slot| *slot < current_slot)
            };

            let Some(target_slot) = target_slot else {
                continue;
            };

            let mut branch_entries = entries
                .iter()
                .filter(|entry| {
                    entry.path.0.len() > level
                        && entry.path.0.get(..level) == Some(parent)
                        && entry.path.0[level] == target_slot
                })
                .collect::<Vec<_>>();
            if branch_entries.is_empty() {
                continue;
            }

            branch_entries.sort_by(|a, b| {
                a.path
                    .0
                    .len()
                    .cmp(&b.path.0.len())
                    .then_with(|| a.path.0.cmp(&b.path.0))
            });
            self.set_focused_entry(branch_entries[0]);
            return true;
        }

        false
    }
}
