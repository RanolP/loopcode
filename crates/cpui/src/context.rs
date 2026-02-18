use std::{
    any::Any,
    cell::RefMut,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::{
    app::App,
    entity::{Entity, EntityId, WeakEntity},
    window::Window,
};

pub struct Context<'a, T: 'static> {
    pub(crate) app: &'a mut App,
    pub(crate) entity: Option<Entity<T>>,
    pub(crate) entity_id: EntityId,
}

impl<'a, T: 'static> Context<'a, T> {
    pub fn entity_id(&self) -> EntityId {
        self.entity_id
    }

    pub fn entity(&self) -> Entity<T> {
        self.entity
            .clone()
            .unwrap_or_else(|| panic!("entity() unavailable while building entity"))
    }

    pub fn weak_entity(&self) -> WeakEntity<T> {
        self.entity().downgrade()
    }

    pub fn notify(&mut self) {}

    pub fn emit<Evt>(&mut self, _event: Evt)
    where
        T: EventEmitter<Evt>,
        Evt: 'static,
    {
    }
}

impl<T: 'static> Deref for Context<'_, T> {
    type Target = App;

    fn deref(&self) -> &Self::Target {
        self.app
    }
}

impl<T: 'static> DerefMut for Context<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.app
    }
}

pub trait EventEmitter<E: Any>: 'static {}
pub trait Focusable {}

pub struct GpuiBorrow<'a, T>(pub(crate) RefMut<'a, T>);

impl<T> Deref for GpuiBorrow<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for GpuiBorrow<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct Reservation<T>(pub(crate) EntityId, pub(crate) PhantomData<T>);

impl<T> Reservation<T> {
    pub fn entity_id(&self) -> EntityId {
        self.0
    }
}

pub trait Global: 'static {}
impl<T: 'static> Global for T {}

pub trait AppContext {
    type Result<T>;

    fn create_entity<T: 'static>(
        &mut self,
        build_entity: impl FnOnce(&mut Context<'_, T>) -> T,
    ) -> Self::Result<Entity<T>>;

    fn reserve_entity<T: 'static>(&mut self) -> Self::Result<Reservation<T>>;

    fn insert_entity<T: 'static>(
        &mut self,
        reservation: Reservation<T>,
        build_entity: impl FnOnce(&mut Context<'_, T>) -> T,
    ) -> Self::Result<Entity<T>>;

    fn update_entity<T: 'static, R>(
        &mut self,
        handle: &Entity<T>,
        update: impl FnOnce(&mut T, &mut Context<'_, T>) -> R,
    ) -> Self::Result<R>;

    fn as_mut<'a, T: 'static>(
        &'a mut self,
        handle: &'a Entity<T>,
    ) -> Self::Result<GpuiBorrow<'a, T>>;

    fn read_entity<T: 'static, R>(
        &self,
        handle: &Entity<T>,
        read: impl FnOnce(&T, &App) -> R,
    ) -> Self::Result<R>;

    fn update_window<T, F>(
        &mut self,
        window: crate::window::AnyWindowHandle,
        f: F,
    ) -> std::io::Result<T>
    where
        F: FnOnce(crate::entity::AnyView, &mut Window, &mut App) -> T;

    fn read_window<T: 'static, R>(
        &self,
        window: &crate::window::WindowHandle<T>,
        read: impl FnOnce(Entity<T>, &App) -> R,
    ) -> std::io::Result<R>;

    fn read_global<G: Global, R>(&self, callback: impl FnOnce(&G, &App) -> R) -> Self::Result<R>;
}

pub trait VisualContext: AppContext {
    fn window_handle(&self) -> crate::window::AnyWindowHandle;

    fn update_window_entity<T: 'static, R>(
        &mut self,
        entity: &Entity<T>,
        update: impl FnOnce(&mut T, &mut Window, &mut Context<'_, T>) -> R,
    ) -> Self::Result<R>;

    fn new_window_entity<T: 'static>(
        &mut self,
        build_entity: impl FnOnce(&mut Window, &mut Context<'_, T>) -> T,
    ) -> Self::Result<Entity<T>>;

    fn replace_root_view<V: 'static + crate::view::Render>(
        &mut self,
        build_view: impl FnOnce(&mut Window, &mut Context<'_, V>) -> V,
    ) -> Self::Result<Entity<V>>;

    fn focus<V: Focusable>(&mut self, _entity: &Entity<V>) -> Self::Result<()>;
}
