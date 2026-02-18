use crate::{context::Context, element::IntoElement, window::Window};

pub trait Render: 'static + Sized {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement;
}
