use crate::{
    node::{Icon, IconName, IntoNode, Node},
    style::Rgb,
};

pub struct IconWidget {
    inner: Icon,
}

impl IconWidget {
    pub fn new(name: IconName) -> Self {
        Self {
            inner: Icon {
                name,
                color: None,
                asset_path: None,
            },
        }
    }

    pub fn color(mut self, color: Rgb) -> Self {
        self.inner.color = Some(color);
        self
    }

    pub fn asset_path(mut self, path: impl Into<String>) -> Self {
        self.inner.asset_path = Some(path.into());
        self
    }
}

impl IntoNode for IconWidget {
    fn into_node(self) -> Node {
        Node::Icon(self.inner)
    }
}

pub fn icon(name: IconName) -> IconWidget {
    IconWidget::new(name)
}
