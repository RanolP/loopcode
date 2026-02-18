use crate::node::{IntoNode, Node};

pub trait Backend {
    type Output;

    fn render_node(&mut self, node: Node) -> Self::Output;
}

pub fn render<B: Backend>(backend: &mut B, root: impl IntoNode) -> B::Output {
    backend.render_node(root.into_node())
}
