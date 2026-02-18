use std::{
    cell::{Cell, Ref, RefCell, RefMut},
    rc::Rc,
    sync::mpsc::{self, Receiver, Sender},
};

#[derive(Clone, Debug)]
pub struct Signal<T> {
    inner: Rc<SignalInner<T>>,
}

#[derive(Debug)]
struct SignalInner<T> {
    value: RefCell<T>,
    version: Cell<u64>,
}

impl<T> Signal<T> {
    pub fn from(value: T) -> Self {
        Self {
            inner: Rc::new(SignalInner {
                value: RefCell::new(value),
                version: Cell::new(0),
            }),
        }
    }

    pub fn version(&self) -> u64 {
        self.inner.version.get()
    }

    pub fn borrow(&self) -> Ref<'_, T> {
        self.inner.value.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        self.bump_version();
        self.inner.value.borrow_mut()
    }

    pub fn set(&self, value: T) {
        *self.inner.value.borrow_mut() = value;
        self.bump_version();
    }

    pub fn update<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut value = self.inner.value.borrow_mut();
        let out = f(&mut value);
        self.bump_version();
        out
    }

    fn bump_version(&self) {
        self.inner.version.set(self.inner.version.get().saturating_add(1));
    }
}

#[derive(Clone, Debug)]
pub struct VecSignal<T> {
    signal: Signal<Vec<T>>,
}

impl<T> VecSignal<T> {
    pub fn from(value: Vec<T>) -> Self {
        Self {
            signal: Signal::from(value),
        }
    }

    pub fn version(&self) -> u64 {
        self.signal.version()
    }

    pub fn borrow(&self) -> Ref<'_, Vec<T>> {
        self.signal.borrow()
    }

    pub fn set(&self, value: Vec<T>) {
        self.signal.set(value);
    }

    pub fn len(&self) -> usize {
        self.signal.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.signal.borrow().is_empty()
    }

    pub fn clear(&self) {
        self.signal.update(|v| v.clear());
    }

    pub fn push(&self, value: T) {
        self.signal.update(|v| v.push(value));
    }

    pub fn pop(&self) -> Option<T> {
        self.signal.update(|v| v.pop())
    }

    pub fn update<R>(&self, f: impl FnOnce(&mut Vec<T>) -> R) -> R {
        self.signal.update(f)
    }
}

pub fn new<T: Default>() -> Signal<T> {
    Signal::from(T::default())
}

pub fn new_vec<T>() -> VecSignal<T> {
    VecSignal::from(Vec::new())
}

#[derive(Debug)]
struct EventSignalInner<T> {
    tx: Sender<T>,
    rx: RefCell<Receiver<T>>,
}

#[derive(Clone, Debug)]
pub struct EventSignal<T> {
    inner: Rc<EventSignalInner<T>>,
}

impl<T> EventSignal<T> {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            inner: Rc::new(EventSignalInner {
                tx,
                rx: RefCell::new(rx),
            }),
        }
    }

    pub fn emit(&self, event: T) {
        let _ = self.inner.tx.send(event);
    }

    pub fn drain(&self, mut on_event: impl FnMut(T)) {
        let rx = self.inner.rx.borrow_mut();
        while let Ok(event) = rx.try_recv() {
            on_event(event);
        }
    }
}

impl<T> Default for EventSignal<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct Memo<K, T> {
    inner: Rc<MemoInner<K, T>>,
}

#[derive(Debug)]
struct MemoInner<K, T> {
    key: RefCell<Option<K>>,
    value: RefCell<Option<T>>,
}

impl<K, T> Memo<K, T>
where
    K: Eq + Clone,
    T: Clone,
{
    pub fn new() -> Self {
        Self {
            inner: Rc::new(MemoInner {
                key: RefCell::new(None),
                value: RefCell::new(None),
            }),
        }
    }

    pub fn get_or_update(&self, key: K, compute: impl FnOnce() -> T) -> T {
        if let Some(current_key) = self.inner.key.borrow().as_ref()
            && *current_key == key
            && let Some(value) = self.inner.value.borrow().as_ref()
        {
            return value.clone();
        }

        let value = compute();
        *self.inner.key.borrow_mut() = Some(key);
        *self.inner.value.borrow_mut() = Some(value.clone());
        value
    }
}

impl<K, T> Default for Memo<K, T>
where
    K: Eq + Clone,
    T: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}
