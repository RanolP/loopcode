use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    io::{self, Write},
    marker::PhantomData,
    rc::Rc,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    time::Duration,
};

use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
    KeyboardEnhancementFlags, MouseEventKind, PopKeyboardEnhancementFlags,
    PushKeyboardEnhancementFlags,
};
use crossterm::execute;
use crossterm::style::ResetColor;
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor, terminal::Clear, terminal::ClearType};

use crate::{
    context::{AppContext, Context, Focusable, Global, GpuiBorrow, Reservation, VisualContext},
    element::IntoElement,
    entity::{AnyEntity, AnyView, Entity, EntityId, WindowId},
    geometry::{Bounds, Pixels, Point, Size},
    view::Render,
    window::{AnyWindowHandle, Window, WindowHandle, WindowOptions},
};

static NEXT_ENTITY_ID: AtomicU64 = AtomicU64::new(1);
static NEXT_WINDOW_ID: AtomicU64 = AtomicU64::new(1);
static ALT_SCREEN_ACTIVE: AtomicBool = AtomicBool::new(false);
const KEYBOARD_FLAGS: KeyboardEnhancementFlags =
    KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
        .union(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
        .union(KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS)
        .union(KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES);

trait WindowRenderer {
    fn render(&self, app: &mut App, window: &mut Window) -> io::Result<()>;
}

pub(crate) fn is_alt_screen_active() -> bool {
    ALT_SCREEN_ACTIVE.load(Ordering::Relaxed)
}

struct ViewRenderer<V: 'static + Render> {
    root: Entity<V>,
}

impl<V: 'static + Render> WindowRenderer for ViewRenderer<V> {
    fn render(&self, app: &mut App, window: &mut Window) -> io::Result<()> {
        let result = app.update_entity(&self.root, |view, cx| {
            let element = view.render(window, cx).into_any_element();
            window.draw(&element)
        });
        result?;
        Ok(())
    }
}

struct NoopRenderer;

impl WindowRenderer for NoopRenderer {
    fn render(&self, _app: &mut App, _window: &mut Window) -> io::Result<()> {
        Ok(())
    }
}

struct WindowState {
    window: Window,
    root: AnyEntity,
    renderer: Box<dyn WindowRenderer>,
}

pub type Result<T> = io::Result<T>;
pub type SharedString = String;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyInput {
    Tab,
    BackTab,
    Left,
    Right,
    WordLeft,
    WordRight,
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
    Backspace,
    BackspaceWord,
    Delete,
    Enter,
    Submit,
    Esc,
    Interrupt,
    Char(char),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputEvent {
    Key(KeyInput),
    ScrollLines(i16),
    AltScreenActive,
}

pub struct App {
    windows: HashMap<WindowId, WindowState>,
    active_window: Option<WindowId>,
    globals: HashMap<TypeId, Box<dyn Any>>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            windows: HashMap::new(),
            active_window: None,
            globals: HashMap::new(),
        }
    }
}

impl App {
    pub fn open_window<V: 'static + Render>(
        &mut self,
        options: WindowOptions,
        build_root_view: impl FnOnce(&mut Window, &mut App) -> Entity<V>,
    ) -> Result<WindowHandle<V>> {
        let id = WindowId(NEXT_WINDOW_ID.fetch_add(1, Ordering::Relaxed));
        let mut window = Window::new(id, options);
        let root = build_root_view(&mut window, self);

        self.windows.insert(
            id,
            WindowState {
                window,
                root: root.clone().into_any(),
                renderer: Box::new(ViewRenderer { root: root.clone() }),
            },
        );
        self.active_window = Some(id);
        self.render_window(id)?;

        Ok(WindowHandle::new(id))
    }

    pub fn activate(&self, _ignoring_other_apps: bool) {}

    pub fn new<T: 'static>(
        &mut self,
        build_entity: impl FnOnce(&mut Context<'_, T>) -> T,
    ) -> Entity<T> {
        <Self as AppContext>::new(self, build_entity)
    }

    pub fn set_global<T: Global>(&mut self, value: T) {
        self.globals.insert(TypeId::of::<T>(), Box::new(value));
    }

    pub fn global<T: Global>(&self) -> Option<&T> {
        self.globals.get(&TypeId::of::<T>())?.downcast_ref::<T>()
    }

    pub fn render_all_windows(&mut self) -> Result<()> {
        let ids: Vec<_> = self.windows.keys().copied().collect();
        for id in ids {
            self.render_window(id)?;
        }
        Ok(())
    }

    fn render_window(&mut self, window_id: WindowId) -> Result<()> {
        let mut state = self
            .windows
            .remove(&window_id)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "window not found"))?;
        state.renderer.render(self, &mut state.window)?;
        self.windows.insert(window_id, state);
        Ok(())
    }
}

impl Bounds {
    pub fn centered(_display: Option<()>, size: Size<Pixels>, _cx: &App) -> Self {
        Self {
            origin: Point::default(),
            size,
        }
    }
}

impl AppContext for App {
    type Result<T> = T;

    fn new<T: 'static>(
        &mut self,
        build_entity: impl FnOnce(&mut Context<'_, T>) -> T,
    ) -> Self::Result<Entity<T>> {
        let id = EntityId(NEXT_ENTITY_ID.fetch_add(1, Ordering::Relaxed));
        let mut cx = Context {
            app: self,
            entity: None,
            entity_id: id,
        };
        let value = build_entity(&mut cx);
        Entity {
            id,
            inner: Rc::new(RefCell::new(value)),
        }
    }

    fn reserve_entity<T: 'static>(&mut self) -> Self::Result<Reservation<T>> {
        Reservation(
            EntityId(NEXT_ENTITY_ID.fetch_add(1, Ordering::Relaxed)),
            PhantomData,
        )
    }

    fn insert_entity<T: 'static>(
        &mut self,
        reservation: Reservation<T>,
        build_entity: impl FnOnce(&mut Context<'_, T>) -> T,
    ) -> Self::Result<Entity<T>> {
        let mut cx = Context {
            app: self,
            entity: None,
            entity_id: reservation.0,
        };
        let value = build_entity(&mut cx);
        Entity {
            id: reservation.0,
            inner: Rc::new(RefCell::new(value)),
        }
    }

    fn update_entity<T: 'static, R>(
        &mut self,
        handle: &Entity<T>,
        update: impl FnOnce(&mut T, &mut Context<'_, T>) -> R,
    ) -> Self::Result<R> {
        let mut entity = handle.inner.borrow_mut();
        let mut cx = Context {
            app: self,
            entity: Some(handle.clone()),
            entity_id: handle.id,
        };
        update(&mut entity, &mut cx)
    }

    fn as_mut<'a, T: 'static>(
        &'a mut self,
        handle: &'a Entity<T>,
    ) -> Self::Result<GpuiBorrow<'a, T>> {
        let _ = self;
        GpuiBorrow(handle.inner.borrow_mut())
    }

    fn read_entity<T: 'static, R>(
        &self,
        handle: &Entity<T>,
        read: impl FnOnce(&T, &App) -> R,
    ) -> Self::Result<R> {
        read(&handle.inner.borrow(), self)
    }

    fn update_window<T, F>(&mut self, window: AnyWindowHandle, f: F) -> io::Result<T>
    where
        F: FnOnce(AnyView, &mut Window, &mut App) -> T,
    {
        let mut state = self
            .windows
            .remove(&window.id)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "window not found"))?;

        let view = AnyView {
            entity: state.root.clone(),
        };
        let out = f(view, &mut state.window, self);
        self.windows.insert(window.id, state);
        Ok(out)
    }

    fn read_window<T: 'static, R>(
        &self,
        window: &WindowHandle<T>,
        read: impl FnOnce(Entity<T>, &App) -> R,
    ) -> io::Result<R> {
        let state = self
            .windows
            .get(&window.id)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "window not found"))?;
        let entity = state
            .root
            .downcast::<T>()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "root type mismatch"))?;
        Ok(read(entity, self))
    }

    fn read_global<G: Global, R>(&self, callback: impl FnOnce(&G, &App) -> R) -> Self::Result<R> {
        let global = self
            .global::<G>()
            .unwrap_or_else(|| panic!("global not set for {}", std::any::type_name::<G>()));
        callback(global, self)
    }
}

impl VisualContext for App {
    fn window_handle(&self) -> AnyWindowHandle {
        AnyWindowHandle {
            id: self.active_window.unwrap_or(WindowId(0)),
        }
    }

    fn update_window_entity<T: 'static, R>(
        &mut self,
        entity: &Entity<T>,
        update: impl FnOnce(&mut T, &mut Window, &mut Context<'_, T>) -> R,
    ) -> Self::Result<R> {
        let active = self.active_window.unwrap_or(WindowId(0));
        let mut state = self.windows.remove(&active).unwrap_or_else(|| WindowState {
            window: Window::new(active, WindowOptions::default()),
            root: entity.clone().into_any(),
            renderer: Box::new(NoopRenderer),
        });

        let out = self.update_entity(entity, |value, cx| update(value, &mut state.window, cx));
        self.windows.insert(active, state);
        out
    }

    fn new_window_entity<T: 'static>(
        &mut self,
        build_entity: impl FnOnce(&mut Window, &mut Context<'_, T>) -> T,
    ) -> Self::Result<Entity<T>> {
        let id = EntityId(NEXT_ENTITY_ID.fetch_add(1, Ordering::Relaxed));
        let mut window = Window::new(WindowId(0), WindowOptions::default());
        let mut cx = Context {
            app: self,
            entity: None,
            entity_id: id,
        };
        let value = build_entity(&mut window, &mut cx);
        Entity {
            id,
            inner: Rc::new(RefCell::new(value)),
        }
    }

    fn replace_root_view<V: 'static + Render>(
        &mut self,
        build_view: impl FnOnce(&mut Window, &mut Context<'_, V>) -> V,
    ) -> Self::Result<Entity<V>> {
        let id = EntityId(NEXT_ENTITY_ID.fetch_add(1, Ordering::Relaxed));
        let active = self.active_window.unwrap_or(WindowId(0));

        let mut state = self.windows.remove(&active).unwrap_or_else(|| WindowState {
            window: Window::new(active, WindowOptions::default()),
            root: AnyEntity {
                id,
                inner: Rc::new(()),
            },
            renderer: Box::new(NoopRenderer),
        });

        let mut cx = Context {
            app: self,
            entity: None,
            entity_id: id,
        };
        let value = build_view(&mut state.window, &mut cx);
        let entity = Entity {
            id,
            inner: Rc::new(RefCell::new(value)),
        };

        state.root = entity.clone().into_any();
        state.renderer = Box::new(ViewRenderer {
            root: entity.clone(),
        });
        self.windows.insert(active, state);
        entity
    }

    fn focus<V: Focusable>(&mut self, _entity: &Entity<V>) -> Self::Result<()> {}
}

pub struct Application {
    headless: bool,
}

impl Application {
    pub fn new() -> Self {
        Self { headless: false }
    }

    pub fn headless() -> Self {
        Self { headless: true }
    }

    pub fn run<F>(self, on_finish_launching: F)
    where
        F: 'static + FnOnce(&mut App),
    {
        self.run_with_input_handler(on_finish_launching, |_app, event| {
            matches!(
                event,
                InputEvent::Key(KeyInput::Char('q')) | InputEvent::Key(KeyInput::Esc)
            )
        });
    }

    pub fn run_with_input_handler<F, H>(self, on_finish_launching: F, mut on_input: H)
    where
        F: 'static + FnOnce(&mut App),
        H: 'static + FnMut(&mut App, InputEvent) -> bool,
    {
        if self.headless {
            let mut app = App::default();
            on_finish_launching(&mut app);
            return;
        }

        if let Err(err) = terminal::enable_raw_mode() {
            eprintln!("cpui raw mode error: {err}");
            return;
        }
        if let Err(err) = execute!(
            io::stdout(),
            EnterAlternateScreen,
            Clear(ClearType::All),
            cursor::MoveTo(0, 0),
            EnableMouseCapture
        ) {
            eprintln!("cpui mouse capture error: {err}");
            let _ = terminal::disable_raw_mode();
            return;
        }
        ALT_SCREEN_ACTIVE.store(true, Ordering::Relaxed);
        let _ = execute!(io::stdout(), PushKeyboardEnhancementFlags(KEYBOARD_FLAGS));
        let terminal_guard = TerminalGuard;
        let mut app = App::default();
        on_finish_launching(&mut app);

        if let Err(err) = app.render_all_windows() {
            eprintln!("cpui render error: {err}");
            return;
        }

        loop {
            if let Ok(true) = event::poll(Duration::from_millis(250)) {
                if let Ok(raw) = event::read() {
                    if matches!(raw, Event::Resize(_, _)) {
                        if let Err(err) = app.render_all_windows() {
                            eprintln!("cpui render error: {err}");
                            break;
                        }
                        continue;
                    }
                    if let Some(input) = map_input_event(raw) {
                        if on_input(&mut app, input) {
                            break;
                        }

                        if let Err(err) = app.render_all_windows() {
                            eprintln!("cpui render error: {err}");
                            break;
                        }
                    }
                }
            }
        }

        drop(terminal_guard);
    }
}

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let mut out = io::stdout();
        let _ = terminal::disable_raw_mode();
        let _ = execute!(
            out,
            DisableMouseCapture,
            PopKeyboardEnhancementFlags,
            ResetColor,
            cursor::Show
        );
        let _ = execute!(out, LeaveAlternateScreen);
        ALT_SCREEN_ACTIVE.store(false, Ordering::Relaxed);
        let _ = out.flush();
    }
}

fn map_input_event(event: Event) -> Option<InputEvent> {
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press => {
            let word_modifier = key
                .modifiers
                .intersects(KeyModifiers::CONTROL | KeyModifiers::SUPER);
            let submit_modifier = key.modifiers.contains(KeyModifiers::ALT);
            match key.code {
                KeyCode::Tab => Some(InputEvent::Key(KeyInput::Tab)),
                KeyCode::BackTab => Some(InputEvent::Key(KeyInput::BackTab)),
                KeyCode::Left if word_modifier => Some(InputEvent::Key(KeyInput::WordLeft)),
                KeyCode::Right if word_modifier => Some(InputEvent::Key(KeyInput::WordRight)),
                KeyCode::Left => Some(InputEvent::Key(KeyInput::Left)),
                KeyCode::Right => Some(InputEvent::Key(KeyInput::Right)),
                KeyCode::Up => Some(InputEvent::Key(KeyInput::Up)),
                KeyCode::Down => Some(InputEvent::Key(KeyInput::Down)),
                KeyCode::PageUp => Some(InputEvent::Key(KeyInput::PageUp)),
                KeyCode::PageDown => Some(InputEvent::Key(KeyInput::PageDown)),
                KeyCode::Home => Some(InputEvent::Key(KeyInput::Home)),
                KeyCode::End => Some(InputEvent::Key(KeyInput::End)),
                KeyCode::Backspace if word_modifier => {
                    Some(InputEvent::Key(KeyInput::BackspaceWord))
                }
                KeyCode::Backspace => Some(InputEvent::Key(KeyInput::Backspace)),
                KeyCode::Char('w' | 'W') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    Some(InputEvent::Key(KeyInput::BackspaceWord))
                }
                KeyCode::Delete => Some(InputEvent::Key(KeyInput::Delete)),
                KeyCode::Enter if submit_modifier => Some(InputEvent::Key(KeyInput::Submit)),
                KeyCode::Enter => Some(InputEvent::Key(KeyInput::Enter)),
                KeyCode::Char('c' | 'C') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    Some(InputEvent::Key(KeyInput::Interrupt))
                }
                KeyCode::Esc => Some(InputEvent::Key(KeyInput::Esc)),
                KeyCode::Char(ch) => Some(InputEvent::Key(KeyInput::Char(ch))),
                _ => None,
            }
        }
        Event::Mouse(mouse) => match mouse.kind {
            MouseEventKind::ScrollUp => Some(InputEvent::ScrollLines(-3)),
            MouseEventKind::ScrollDown => Some(InputEvent::ScrollLines(3)),
            _ => None,
        },
        _ => None,
    }
}
