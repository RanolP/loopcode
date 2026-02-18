use crate::{
    node::{FocusId, IntoNode, Node, TextInput},
    runtime::TextInputState,
};

pub struct TextInputWidget {
    inner: TextInput,
}

impl TextInputWidget {
    pub fn new(value: impl Into<String>) -> Self {
        let value = value.into();
        let cursor = value.chars().count();
        Self {
            inner: TextInput {
                focus_id: None,
                value,
                placeholder: None,
                cursor,
                focused: false,
                visible_offset_lines: 0,
            },
        }
    }

    pub fn from_state(state: &TextInputState) -> Self {
        Self {
            inner: TextInput {
                focus_id: None,
                value: state.value().to_string(),
                placeholder: None,
                cursor: state.cursor(),
                focused: false,
                visible_offset_lines: 0,
            },
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.inner.placeholder = Some(placeholder.into());
        self
    }

    pub fn cursor(mut self, cursor: usize) -> Self {
        self.inner.cursor = cursor;
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.inner.focused = focused;
        self
    }

    pub fn focus(mut self, focus_id: FocusId) -> Self {
        self.inner.focus_id = Some(focus_id);
        self
    }

    pub fn visible_offset_lines(mut self, lines: u16) -> Self {
        self.inner.visible_offset_lines = lines;
        self
    }
}

impl IntoNode for TextInputWidget {
    fn into_node(self) -> Node {
        Node::TextInput(self.inner)
    }
}

pub fn text_input(value: impl Into<String>) -> TextInputWidget {
    TextInputWidget::new(value)
}

pub fn text_input_from_state(state: &TextInputState) -> TextInputWidget {
    TextInputWidget::from_state(state)
}
