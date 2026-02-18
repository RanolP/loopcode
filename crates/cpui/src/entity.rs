use std::{
    any::Any,
    cell::RefCell,
    rc::{Rc, Weak},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EntityId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WindowId(pub u64);

pub struct Entity<T: 'static> {
    pub(crate) id: EntityId,
    pub(crate) inner: Rc<RefCell<T>>,
}

impl<T: 'static> Clone for Entity<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            inner: self.inner.clone(),
        }
    }
}

impl<T: 'static> Entity<T> {
    pub fn entity_id(&self) -> EntityId {
        self.id
    }

    pub fn downgrade(&self) -> WeakEntity<T> {
        WeakEntity {
            id: self.id,
            inner: Rc::downgrade(&self.inner),
        }
    }

    pub(crate) fn into_any(&self) -> AnyEntity {
        AnyEntity {
            id: self.id,
            inner: Rc::new(self.clone()),
        }
    }
}

pub struct WeakEntity<T: 'static> {
    pub(crate) id: EntityId,
    pub(crate) inner: Weak<RefCell<T>>,
}

impl<T: 'static> Clone for WeakEntity<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            inner: self.inner.clone(),
        }
    }
}

impl<T: 'static> WeakEntity<T> {
    pub fn upgrade(&self) -> Option<Entity<T>> {
        self.inner
            .upgrade()
            .map(|inner| Entity { id: self.id, inner })
    }
}

#[derive(Clone)]
pub struct AnyEntity {
    pub(crate) id: EntityId,
    pub(crate) inner: Rc<dyn Any>,
}

impl AnyEntity {
    pub fn entity_id(&self) -> EntityId {
        self.id
    }

    pub fn downcast<T: 'static>(&self) -> Option<Entity<T>> {
        self.inner.downcast_ref::<Entity<T>>().cloned()
    }
}

#[derive(Clone)]
pub struct AnyView {
    pub(crate) entity: AnyEntity,
}

impl AnyView {
    pub fn entity_id(&self) -> EntityId {
        self.entity.entity_id()
    }
}
