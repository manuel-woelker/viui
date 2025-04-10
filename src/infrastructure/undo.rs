use std::fmt::Debug;

/**
 * https://github.com/zaboople/klonk/blob/master/TheGURQ.md
*/
pub struct UndoLogic<ACTION: Debug> {
    undo_stack: Vec<UndoStackEntry<ACTION>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DoOrUndo<'a, ACTION> {
    Do(&'a ACTION),
    Undo(&'a ACTION),
}

impl<'a, ACTION> DoOrUndo<'a, ACTION> {
    fn invert(&mut self) {
        *self = match *self {
            DoOrUndo::Do(action) => DoOrUndo::Undo(action),
            DoOrUndo::Undo(action) => DoOrUndo::Do(action),
        }
    }
}
pub type ActionVec<'a, ACTION> = Vec<DoOrUndo<'a, ACTION>>;

#[derive(Debug, PartialEq, Eq)]
pub struct UndoInfo<'a, ACTION> {
    pub actions: ActionVec<'a, ACTION>,
}

impl<'a, ACTION> UndoInfo<'a, ACTION> {
    pub fn new() -> Self {
        Self { actions: vec![] }
    }

    pub fn new_do(action: &'a ACTION) -> Self {
        Self {
            actions: vec![DoOrUndo::Do(action)],
        }
    }

    pub fn new_undo(action: &'a ACTION) -> Self {
        Self {
            actions: vec![DoOrUndo::Undo(action)],
        }
    }

    pub(crate) fn push_do(&mut self, action: &'a ACTION) {
        self.actions.push(DoOrUndo::Do(action));
    }
    pub(crate) fn push_undo(&mut self, action: &'a ACTION) {
        self.actions.push(DoOrUndo::Undo(action));
    }

    pub(crate) fn invert(&mut self) {
        self.actions.reverse();
        self.actions
            .iter_mut()
            .for_each(|do_or_undo: &mut DoOrUndo<ACTION>| {
                do_or_undo.invert();
            });
    }
}

#[derive(Debug)]
enum UndoStackEntry<ACTION> {
    Action { label: String, action: ACTION },
    Undos { how_many: usize },
}

impl<ACTION> UndoStackEntry<ACTION> {
    pub fn label(&self) -> Option<&str> {
        match self {
            UndoStackEntry::Action { label, .. } => Some(label),
            _ => None,
        }
    }
    pub fn action(&self) -> Option<&ACTION> {
        match self {
            UndoStackEntry::Action { action, .. } => Some(action),
            _ => None,
        }
    }
}

impl<ACTION: Debug> UndoLogic<ACTION> {
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

    pub fn undo(&mut self) -> UndoInfo<ACTION> {
        let mut result = UndoInfo::new();
        let stack_len = self.undo_stack.len();
        let offset = match self.undo_stack.last_mut() {
            Some(UndoStackEntry::Action { .. }) => Some(1),
            Some(UndoStackEntry::Undos { how_many }) => {
                if *how_many + 1 >= stack_len {
                    // no more undos
                    return result;
                }
                *how_many += 1;
                Some(*how_many as usize)
            }
            _ => None,
        };
        if Some(1) == offset {
            self.undo_stack.push(UndoStackEntry::Undos { how_many: 1 });
        }
        if let Some(offset) = offset {
            if offset >= self.undo_stack.len() {
                return result;
            }
            let position = self.undo_stack.len() - 1 - offset;
            let entry = self.undo_stack.get(position);
            match entry {
                Some(UndoStackEntry::Action { action, .. }) => {
                    result.push_undo(action);
                }
                Some(UndoStackEntry::Undos { how_many }) => {
                    self.collect_actions(position, *how_many, &mut result);
                    result.invert();
                }
                None => {}
            }
        }
        result
    }

    fn collect_actions<'a, 'b: 'a>(
        &'b self,
        position: usize,
        mut how_many: usize,
        undo_info: &'a mut UndoInfo<'b, ACTION>,
    ) {
        let mut index = position - 1;
        loop {
            if how_many == 0 {
                break;
            }
            how_many -= 1;
            let entry = &self.undo_stack[index];
            match entry {
                UndoStackEntry::Action { action, .. } => {
                    index -= 1;
                    undo_info.push_undo(action);
                }
                UndoStackEntry::Undos { how_many } => {
                    let mut sub_result = UndoInfo::new();
                    self.collect_actions(index, *how_many, &mut sub_result);
                    sub_result.invert();
                    index -= sub_result.actions.len() + 1;
                    undo_info.actions.append(&mut sub_result.actions);
                }
            }
        }
    }

    fn get_previous_actions(&self, position: usize, how_many: usize) -> Vec<&ACTION> {
        let mut result = vec![];
        for index in position - how_many..position {
            result.push(self.undo_stack[index].action().unwrap());
        }
        result
    }

    pub fn redo(&mut self) -> UndoInfo<ACTION> {
        let mut result = UndoInfo::new();
        let offset = match self.undo_stack.last_mut() {
            Some(UndoStackEntry::Action { .. }) => None,
            Some(UndoStackEntry::Undos { how_many }) => {
                *how_many -= 1;
                Some(*how_many)
            }
            _ => None,
        };
        let undo_stack_len = self.undo_stack.len();
        if let Some(offset) = offset {
            if offset == 0 {
                self.undo_stack.pop();
            }
            if offset >= undo_stack_len {
                return result;
            }
            let position = undo_stack_len - 2 - offset;
            let entry = self.undo_stack.get(position);
            match entry {
                Some(UndoStackEntry::Action { action, .. }) => {
                    result.push_do(action);
                }
                Some(UndoStackEntry::Undos { how_many }) => {
                    self.collect_actions(position, *how_many, &mut result);
                }
                None => {}
            }
        }
        result
    }

    pub fn get_undo_label(&self) -> Option<&str> {
        match self.undo_stack.last() {
            Some(entry) => entry.label(),
            _ => None,
        }
    }
    pub fn get_redo_label(&self) -> Option<&str> {
        match self.undo_stack.last() {
            Some(UndoStackEntry::Action { .. }) => None,
            Some(UndoStackEntry::Undos { how_many }) => self
                .undo_stack
                .get(self.undo_stack.len() - 1 - *how_many as usize)
                .and_then(|entry| entry.label()),
            _ => None,
        }
    }

    pub(crate) fn get_undo_stack(&self) -> &[UndoStackEntry<ACTION>] {
        &self.undo_stack
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct Increment(i32);

    fn get_stack_string(undo: &UndoLogic<Increment>) -> String {
        let mut result = String::new();
        for entry in undo.get_undo_stack() {
            match entry {
                UndoStackEntry::Action {
                    action: Increment(i),
                    ..
                } => {
                    result += &format!("i{i} ");
                }
                UndoStackEntry::Undos { how_many } => {
                    result += &format!("u{how_many} ");
                }
            }
        }
        result
    }

    #[test]
    fn new() {
        let mut undo = UndoLogic::<Increment>::new();
        assert_eq!(undo.get_undo_label(), None);
        assert_eq!(undo.get_redo_label(), None);
        assert_eq!(get_stack_string(&undo), "");
        let undo_info = undo.undo();
        assert_eq!(undo_info, UndoInfo::new());
    }

    #[test]
    fn simple_undo() {
        let mut undo = UndoLogic::<Increment>::new();
        undo.push_action("test", Increment(1));
        assert_eq!(get_stack_string(&undo), "i1 ");
        assert_eq!(undo.get_undo_label(), Some("test"));
        assert_eq!(undo.get_redo_label(), None);

        let undo_info = undo.undo();
        assert_eq!(undo_info, UndoInfo::new_undo(&Increment(1)));

        assert_eq!(undo.get_redo_label(), Some("test"));
        assert_eq!(undo.get_undo_label(), None);

        assert_eq!(get_stack_string(&undo), "i1 u1 ");

        assert_eq!(undo.undo(), UndoInfo::new());
    }

    #[test]
    fn double_undo() {
        let mut undo = UndoLogic::<Increment>::new();
        undo.push_action("a", Increment(1));
        undo.push_action("b", Increment(2));

        let undo_info = undo.undo();
        assert_eq!(undo_info, UndoInfo::new_undo(&Increment(2)));
        assert_eq!(get_stack_string(&undo), "i1 i2 u1 ");

        let undo_info = undo.undo();
        assert_eq!(undo_info, UndoInfo::new_undo(&Increment(1)));
        assert_eq!(get_stack_string(&undo), "i1 i2 u2 ");

        assert_eq!(undo.undo(), UndoInfo::new());
        assert_eq!(get_stack_string(&undo), "i1 i2 u2 ");

        let undo_info = undo.redo();
        assert_eq!(undo_info, UndoInfo::new_do(&Increment(1)));
        assert_eq!(get_stack_string(&undo), "i1 i2 u1 ");

        let undo_info = undo.redo();
        assert_eq!(undo_info, UndoInfo::new_do(&Increment(2)));
        assert_eq!(get_stack_string(&undo), "i1 i2 ");

        assert_eq!(undo.redo(), UndoInfo::new());
    }

    #[test]
    fn fork_and_redo() {
        let mut undo = UndoLogic::<Increment>::new();
        undo.push_action("a", Increment(1));
        undo.push_action("b", Increment(2));

        let _undo_info = undo.undo();
        undo.push_action("c", Increment(3));

        assert_eq!(get_stack_string(&undo), "i1 i2 u1 i3 ");
    }

    #[test]
    fn undo_an_undo() {
        let mut undo = UndoLogic::<Increment>::new();
        undo.push_action("a", Increment(1));
        undo.push_action("b", Increment(2));

        let _undo_info = undo.undo();
        undo.push_action("c", Increment(3));

        assert_eq!(get_stack_string(&undo), "i1 i2 u1 i3 ");
        let undo_info = undo.undo();
        assert_eq!(undo_info, UndoInfo::new_undo(&Increment(3)));
        assert_eq!(get_stack_string(&undo), "i1 i2 u1 i3 u1 ");

        let undo_info = undo.undo();
        assert_eq!(undo_info, UndoInfo::new_do(&Increment(2)));
        assert_eq!(get_stack_string(&undo), "i1 i2 u1 i3 u2 ");
    }

    #[test]
    fn undo_a_double_undo() {
        let mut undo = UndoLogic::<Increment>::new();
        undo.push_action("a", Increment(1));
        undo.push_action("b", Increment(2));
        undo.push_action("c", Increment(3));

        let _undo_info = undo.undo();
        let _undo_info = undo.undo();
        undo.push_action("c", Increment(4));

        assert_eq!(get_stack_string(&undo), "i1 i2 i3 u2 i4 ");
        let undo_info = undo.undo();
        assert_eq!(undo_info, UndoInfo::new_undo(&Increment(4)));
        assert_eq!(get_stack_string(&undo), "i1 i2 i3 u2 i4 u1 ");

        let undo_info = undo.undo();
        let mut expected_undo_info = UndoInfo::new_do(&Increment(2));
        expected_undo_info.push_do(&Increment(3));
        assert_eq!(undo_info, expected_undo_info);
        assert_eq!(get_stack_string(&undo), "i1 i2 i3 u2 i4 u2 ");

        let undo_info = undo.redo();
        let mut expected_undo_info = UndoInfo::new_undo(&Increment(3));
        expected_undo_info.push_undo(&Increment(2));
        assert_eq!(undo_info, expected_undo_info);
        assert_eq!(get_stack_string(&undo), "i1 i2 i3 u2 i4 u1 ");
    }

    #[test]
    fn triple_undo() {
        let mut undo = UndoLogic::<Increment>::new();
        undo.push_action("a", Increment(1));
        undo.push_action("b", Increment(2));
        undo.push_action("c", Increment(3));

        let _undo_info = undo.undo();
        let _undo_info = undo.undo();
        undo.push_action("c", Increment(4));

        let _undo_info = undo.undo();
        let _undo_info = undo.undo();

        assert_eq!(get_stack_string(&undo), "i1 i2 i3 u2 i4 u2 ");
        undo.push_action("c", Increment(5));
        assert_eq!(get_stack_string(&undo), "i1 i2 i3 u2 i4 u2 i5 ");

        let undo_info = undo.undo();
        assert_eq!(undo_info, UndoInfo::new_undo(&Increment(5)));
        assert_eq!(get_stack_string(&undo), "i1 i2 i3 u2 i4 u2 i5 u1 ");

        let undo_info = undo.undo();
        let mut expected_undo_info = UndoInfo::new_undo(&Increment(3));
        expected_undo_info.push_undo(&Increment(2));
        expected_undo_info.push_do(&Increment(4));
        assert_eq!(undo_info, expected_undo_info);
        assert_eq!(get_stack_string(&undo), "i1 i2 i3 u2 i4 u2 i5 u2 ");
    }
}
