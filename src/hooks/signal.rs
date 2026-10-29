use std::{fmt, marker::PhantomData};

use gpui_kit::{App, EntityId, Global, Window};

/// A `Copy` handle to a value owned by the app.
///
/// Signals are what [`use_state`](crate::hooks::use_state) returns. Any closure can capture
/// one without cloning. Read with [`get`](Self::get) / [`read`](Self::read) and write with
/// [`set`](Self::set) / [`update`](Self::update), which re-renders the component that created it.
///
/// Inside a `#[component]`, `count()` reads, `count(5)` sets and `count(|n| *n += 1)` updates;
/// the macro passes `cx` for you.
///
/// Signals are deliberately not `PartialEq`: compare values (`count() == 1`), not handles.
pub struct Signal<T> {
    index: u32,
    generation: u32,
    _value: PhantomData<fn() -> T>,
}


struct Slot<T> {
    generation: u32,
    /// bumped on every write, so readers can tell whether the value changed
    version: u32,
    view: Option<EntityId>,
    value: Option<T>,
}

/// One slot table per value type, so a lookup is an index: no boxing, no downcasting.
struct Store<T> {
    slots: Vec<Slot<T>>,
    free: Vec<u32>,
}

impl<T> Default for Store<T> {
    fn default() -> Self {
        Self {
            slots: Vec::new(),
            free: Vec::new(),
        }
    }
}

impl<T: 'static> Global for Store<T> {}

impl<T: 'static> Signal<T> {
    /// Creates a signal that re-renders `view` (if any) when written.
    /// It lives until it is [`dispose`](Self::dispose)d; hooks do that when their component unmounts.
    pub fn new(value: T, view: Option<EntityId>, cx: &mut App) -> Self {
        let store = cx.default_global::<Store<T>>();
        let index = store.free.pop().unwrap_or_else(|| {
            store.slots.push(Slot {
                generation: 0,
                version: 0,
                view: None,
                value: None,
            });
            store.slots.len() as u32 - 1
        });
        let slot = &mut store.slots[index as usize];
        slot.view = view;
        slot.value = Some(value);
        Self {
            index,
            generation: slot.generation,
            _value: PhantomData,
        }
    }

    fn slot(self, cx: &App) -> Option<&Slot<T>> {
        let slot = cx.try_global::<Store<T>>()?.slots.get(self.index as usize)?;
        (slot.generation == self.generation && slot.value.is_some()).then_some(slot)
    }

    /// Borrows the value, or `None` if the signal was disposed.
    pub fn try_read(self, cx: &App) -> Option<&T> {
        self.slot(cx)?.value.as_ref()
    }

    /// Borrows the value.
    ///
    /// # Panics
    ///
    /// Panics if the component that owned the signal has unmounted.
    #[track_caller]
    pub fn read(self, cx: &App) -> &T {
        self.try_read(cx)
            .expect("signal read after its component unmounted")
    }

    /// Returns a copy of the value.
    #[track_caller]
    pub fn get(self, cx: &App) -> T
    where
        T: Clone,
    {
        self.read(cx).clone()
    }

    /// Replaces the value and re-renders its component, unless the value is unchanged.
    /// Use [`update`](Self::update) for types that aren't `PartialEq` or are costly to compare.
    pub fn set(self, value: T, cx: &mut App)
    where
        T: PartialEq,
    {
        if self.try_read(cx).is_some_and(|current| *current != value) {
            self.update(|current| *current = value, cx);
        }
    }

    /// Mutates the value in place and re-renders its component.
    /// Does nothing if the signal was disposed.
    pub fn update<R>(self, f: impl FnOnce(&mut T) -> R, cx: &mut App) {
        self.try_update(f, cx);
    }

    /// Like [`update`](Self::update), but returns `f`'s result, or `None` if the signal was
    /// disposed (e.g. a task that outlived its component can stop on `None`).
    pub fn try_update<R>(self, f: impl FnOnce(&mut T) -> R, cx: &mut App) -> Option<R> {
        let (result, view) = self.write(f, cx)?;
        if let Some(view) = view {
            cx.notify(view);
        }
        Some(result)
    }

    /// Mutates the value without re-rendering anything.
    pub fn update_untracked<R>(self, f: impl FnOnce(&mut T) -> R, cx: &mut App) {
        self.write(f, cx);
    }

    /// Drops the value. Later reads panic and later writes are ignored.
    pub fn dispose(self, cx: &mut App) {
        let store = cx.global_mut::<Store<T>>();
        if let Some(slot) = store.slots.get_mut(self.index as usize)
            && slot.generation == self.generation
        {
            slot.value = None;
            // a slot whose generation would wrap is retired, so a stale handle can never match again
            if let Some(generation) = slot.generation.checked_add(1) {
                slot.generation = generation;
                store.free.push(self.index);
            }
        }
    }

    /// Whether both handles point at the same signal.
    pub(crate) fn same(self, other: Self) -> bool {
        (self.index, self.generation) == (other.index, other.generation)
    }

    /// Identifies this signal: unique among live signals of `T` and never reused by a later one.
    pub fn id(self) -> u64 {
        (u64::from(self.generation) << 32) | u64::from(self.index)
    }

    /// Changes whenever the value is written; `None` once disposed.
    pub fn version(self, cx: &App) -> Option<u32> {
        Some(self.slot(cx)?.version)
    }

    fn write<R>(self, f: impl FnOnce(&mut T) -> R, cx: &mut App) -> Option<(R, Option<EntityId>)> {
        let slot = cx
            .global_mut::<Store<T>>()
            .slots
            .get_mut(self.index as usize)
            .filter(|slot| slot.generation == self.generation)?;
        let result = f(slot.value.as_mut()?);
        slot.version = slot.version.wrapping_add(1);
        Some((result, slot.view))
    }
}

/// Creates a signal owned by the current component: it is created on the first render,
/// the same handle is returned after that, and it is disposed when the component unmounts.
///
/// # Example
///
/// ```ignore
/// let (count, set_count) = use_signal(0);
/// set_count(count() + 1);
/// ```
#[track_caller]
pub fn use_signal<T: 'static>(
    init: impl FnOnce() -> T,
    window: &mut Window,
    cx: &mut App,
) -> Signal<T> {
    let view = window.current_view();
    *window
        .use_state(cx, |_window, cx| {
            let signal = Signal::new(init(), Some(view), cx);
            cx.on_release(move |_, cx| signal.dispose(cx)).detach();
            signal
        })
        .read(cx)
}

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Signal<T> {}

impl<T> fmt::Debug for Signal<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Signal({}v{})", self.index, self.generation)
    }
}
