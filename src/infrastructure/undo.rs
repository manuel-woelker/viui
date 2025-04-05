/**
 * https://github.com/zaboople/klonk/blob/master/TheGURQ.md
*/
pub struct UndoLogic<ACTION> {
    undo_stack: Vec<UndoStackEntry<ACTION>>,
}

enum UndoStackEntry<ACTION> {
    Action { label: String, action: ACTION },
    Undos { how_many: u32 },
}

impl<ACTION> UndoStackEntry<ACTION> {
    pub fn label(&self) -> Option<&str> {
        match self {
            UndoStackEntry::Action { label, .. } => Some(label),
            _ => None,
        }
    }
}

impl<ACTION> UndoLogic<ACTION> {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
        }
    }

    pub fn push_action<S: ToString>(&mut self, label: S, action: ACTION) {
        self.undo_stack.push(UndoStackEntry::Action {
            label: label.to_string(),
            action,
        })
    }

    pub fn undo(&mut self) -> Option<&ACTION> {
        let index = match self.undo_stack.last() {
            Some(UndoStackEntry::Action { action, .. }) => Some(self.undo_stack.len() - 1),
            _ => None,
        };

        self.undo_stack.push(UndoStackEntry::Undos { how_many: 1 });
        if let Some(index) = index {
            self.undo_stack.get(index).and_then(|entry| match entry {
                UndoStackEntry::Action { action, .. } => Some(action),
                _ => None,
            })
        } else {
            None
        }
    }

    pub fn get_undo_label(&self) -> Option<&str> {
        match self.undo_stack.last() {
            Some(entry) => entry.label(),
            _ => None,
        }
    }
    pub fn get_redo_label(&self) -> Option<&str> {
        dbg!(self.undo_stack.last().and_then(|e| e.label()));
        match self.undo_stack.last() {
            Some(UndoStackEntry::Action { .. }) => None,
            Some(UndoStackEntry::Undos { how_many }) => self
                .undo_stack
                .get(self.undo_stack.len() - 1 - *how_many as usize)
                .and_then(|entry| entry.label()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct Increment(i32);

    #[test]
    fn new() {
        let undo = UndoLogic::<()>::new();
        assert_eq!(undo.get_undo_label(), None);
        assert_eq!(undo.get_redo_label(), None);
    }

    #[test]
    fn simple_undo() {
        let mut undo = UndoLogic::<Increment>::new();
        undo.push_action("test", Increment(1));
        assert_eq!(undo.get_undo_label(), Some("test"));
        assert_eq!(undo.get_redo_label(), None);
        let undo_info = undo.undo();
        assert_eq!(undo_info, Some(&Increment(1)));

        assert_eq!(undo.get_redo_label(), Some("test"));
        assert_eq!(undo.get_undo_label(), None);
    }
}
