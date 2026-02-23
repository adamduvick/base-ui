//! Pub/sub store with observer pattern.
//!
//! Ported from `@base-ui/utils/store`.
//! Provides a `Store<State>` that holds state, notifies listeners on changes,
//! and supports selective observation via selector functions.
//!
//! ## What's ported
//! - `Store` — core pub/sub store with subscribe, setState, update, set, observe
//! - `create_selector` — simple selector composition (non-memoized)
//!
//! ## What's skipped (React-specific)
//! - `ReactStore` — extends Store with React hooks (useSyncedValue, useControlledProp, etc.)
//! - `useStore` — React hook using useSyncExternalStore
//! - `StoreInspector` — React debugging component
//! - `createSelectorMemoized` — depends on `reselect` (a JS library)
//!
//! In Leptos, for fine-grained reactivity, prefer using Leptos signals (`RwSignal`)
//! directly. This `Store` is provided for component internals that need the same
//! pub/sub API as the React version (e.g., compound component state sharing).

use std::cell::{Cell, RefCell};
use std::rc::Rc;

type Listener<State> = Box<dyn Fn(&State)>;

/// A data store with an observer pattern for state change notifications.
///
/// Maps to the `Store` class in the React version.
///
/// # Type Parameters
/// * `State` — The state type. Must be `Clone` and `PartialEq` for change detection.
pub struct Store<State> {
    state: RefCell<State>,
    listeners: RefCell<Vec<Option<Listener<State>>>>,
    update_tick: Cell<u64>,
    next_listener_id: Cell<usize>,
}

impl<State> Store<State>
where
    State: Clone + PartialEq + 'static,
{
    /// Create a new store with the given initial state.
    pub fn new(initial_state: State) -> Rc<Self> {
        Rc::new(Store {
            state: RefCell::new(initial_state),
            listeners: RefCell::new(Vec::new()),
            update_tick: Cell::new(0),
            next_listener_id: Cell::new(0),
        })
    }

    /// Get a clone of the current state.
    pub fn get_state(&self) -> State {
        self.state.borrow().clone()
    }

    /// Get a reference to the current state (for use within callbacks).
    pub fn with_state<R>(&self, f: impl FnOnce(&State) -> R) -> R {
        f(&self.state.borrow())
    }

    /// Subscribe to state changes. Returns an unsubscribe function.
    ///
    /// The listener is called with a reference to the new state whenever
    /// `set_state`, `update`, or `set` causes a change.
    pub fn subscribe(self: &Rc<Self>, listener: impl Fn(&State) + 'static) -> impl FnOnce() {
        let id = self.next_listener_id.get();
        self.next_listener_id.set(id + 1);

        let mut listeners = self.listeners.borrow_mut();
        // Ensure the Vec is large enough
        if id >= listeners.len() {
            listeners.resize_with(id + 1, || None);
        }
        listeners[id] = Some(Box::new(listener));

        let store = Rc::clone(self);
        move || {
            let mut listeners = store.listeners.borrow_mut();
            if id < listeners.len() {
                listeners[id] = None;
            }
        }
    }

    /// Replace the entire state and notify listeners.
    ///
    /// No-op if the new state equals the current state.
    pub fn set_state(&self, new_state: State) {
        {
            let current = self.state.borrow();
            if *current == new_state {
                return;
            }
        }

        *self.state.borrow_mut() = new_state;
        self.update_tick.set(self.update_tick.get() + 1);

        let current_tick = self.update_tick.get();
        let listeners = self.listeners.borrow();
        for listener in listeners.iter().flatten() {
            if current_tick != self.update_tick.get() {
                // A recursive set_state was called; it already notified all listeners.
                return;
            }
            listener(&self.state.borrow());
        }
    }

    /// Notify all listeners unconditionally by cloning the state to create
    /// a new reference.
    ///
    /// Maps to `notifyAll()` in the React version.
    pub fn notify_all(&self) {
        let new_state = self.state.borrow().clone();
        // Bypass the equality check by incrementing the tick directly
        self.update_tick.set(self.update_tick.get() + 1);
        *self.state.borrow_mut() = new_state;

        let current_tick = self.update_tick.get();
        let listeners = self.listeners.borrow();
        for listener in listeners.iter().flatten() {
            if current_tick != self.update_tick.get() {
                return;
            }
            listener(&self.state.borrow());
        }
    }

    /// Observe a derived value from the store. The `selector` function extracts
    /// a value from the state. The `listener` is called whenever the selected
    /// value changes (compared with `PartialEq`).
    ///
    /// The listener is called immediately with the current value on subscription.
    ///
    /// Returns an unsubscribe function.
    ///
    /// Maps to `store.observe(selector, listener)` in the React version.
    pub fn observe<V>(
        self: &Rc<Self>,
        selector: impl Fn(&State) -> V + 'static,
        listener: impl Fn(&V, &V) + 'static,
    ) -> impl FnOnce()
    where
        V: PartialEq + Clone + 'static,
    {
        let prev_value = Rc::new(RefCell::new(selector(&self.state.borrow())));

        // Call listener immediately with current value
        {
            let current = prev_value.borrow();
            listener(&current, &current);
        }

        let prev_for_sub = prev_value.clone();
        self.subscribe(move |new_state| {
            let new_value = selector(new_state);
            let old_value = prev_for_sub.borrow().clone();
            if new_value != old_value {
                *prev_for_sub.borrow_mut() = new_value.clone();
                listener(&new_value, &old_value);
            }
        })
    }
}

impl<State> Store<State>
where
    State: Clone + PartialEq + UpdateFields + 'static,
{
    /// Merge partial changes into the current state.
    ///
    /// Only triggers an update if at least one field actually changed.
    ///
    /// Maps to `store.update(changes)` in the React version.
    /// Requires implementing `UpdateFields` on the state type.
    pub fn update_fields(&self, apply: impl FnOnce(&mut State)) {
        let mut new_state = self.state.borrow().clone();
        apply(&mut new_state);
        self.set_state(new_state);
    }
}

/// Trait for state types that support partial field updates.
/// Implement this on your state struct to use `Store::update_fields`.
pub trait UpdateFields {}

/// Create a simple composed selector function.
///
/// Takes two functions: an extractor and a combiner. The extractor selects
/// a value from the state, and the combiner transforms it.
///
/// Maps to `createSelector(extractor, combiner)` in the React version (simple form).
///
/// For Leptos, prefer using `create_memo` for memoized selectors.
pub fn create_selector<State, Intermediate, Output>(
    extractor: impl Fn(&State) -> Intermediate,
    combiner: impl Fn(Intermediate) -> Output,
) -> impl Fn(&State) -> Output {
    move |state| combiner(extractor(state))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Ported from ReactStore.test.tsx:
    // ✓ Store basic set_state and get_state
    // ✓ Store subscribe and notification
    // ✓ Store set_state no-op on equal state
    // ✓ Store notify_all
    // ✓ Store observe calls listener immediately
    // ✓ Store observe calls listener on change
    // ✓ Store observe does not call listener when unchanged
    // ✓ Store observe unsubscribe stops notifications
    // ✓ Store multiple observers
    // ✓ create_selector composes functions
    // ✗ useControlledProp (React-specific)
    // ✗ useSyncedValue (React-specific)
    // ✗ useSyncedValues (React-specific)
    // ✗ useSyncedValueWithCleanup (React-specific)
    // ✗ useStateSetter (React-specific)
    // ✗ useState (React-specific, uses useSyncExternalStore)
    // ✗ useContextCallback (React-specific)

    #[derive(Clone, Debug, PartialEq)]
    struct TestState {
        count: i32,
        label: String,
        multiplier: i32,
    }

    impl UpdateFields for TestState {}

    #[test]
    fn basic_get_set_state() {
        let store = Store::new(TestState {
            count: 0,
            label: String::new(),
            multiplier: 1,
        });
        assert_eq!(store.get_state().count, 0);

        store.set_state(TestState {
            count: 5,
            label: String::new(),
            multiplier: 1,
        });
        assert_eq!(store.get_state().count, 5);
    }

    #[test]
    fn subscribe_and_notification() {
        let store = Store::new(TestState {
            count: 0,
            label: String::new(),
            multiplier: 1,
        });

        let calls = Rc::new(RefCell::new(Vec::new()));
        let calls_clone = calls.clone();
        let _unsub = store.subscribe(move |state| {
            calls_clone.borrow_mut().push(state.count);
        });

        store.set_state(TestState {
            count: 1,
            label: String::new(),
            multiplier: 1,
        });
        store.set_state(TestState {
            count: 2,
            label: String::new(),
            multiplier: 1,
        });

        assert_eq!(*calls.borrow(), vec![1, 2]);
    }

    #[test]
    fn no_notification_on_equal_state() {
        let store = Store::new(TestState {
            count: 5,
            label: String::new(),
            multiplier: 1,
        });

        let call_count = Rc::new(Cell::new(0));
        let cc = call_count.clone();
        let _unsub = store.subscribe(move |_| {
            cc.set(cc.get() + 1);
        });

        store.set_state(TestState {
            count: 5,
            label: String::new(),
            multiplier: 1,
        });
        assert_eq!(call_count.get(), 0);
    }

    #[test]
    fn notify_all_fires_even_without_change() {
        let store = Store::new(TestState {
            count: 5,
            label: String::new(),
            multiplier: 1,
        });

        let call_count = Rc::new(Cell::new(0));
        let cc = call_count.clone();
        let _unsub = store.subscribe(move |_| {
            cc.set(cc.get() + 1);
        });

        store.notify_all();
        assert_eq!(call_count.get(), 1);
    }

    #[test]
    fn observe_calls_listener_immediately() {
        let store = Store::new(TestState {
            count: 5,
            label: String::new(),
            multiplier: 3,
        });

        let calls: Rc<RefCell<Vec<(i32, i32)>>> = Rc::new(RefCell::new(Vec::new()));
        let calls_clone = calls.clone();

        let _unsub = store.observe(
            |s| s.count * 2,
            move |new_val, old_val| {
                calls_clone.borrow_mut().push((*new_val, *old_val));
            },
        );

        // Called immediately with (10, 10)
        assert_eq!(*calls.borrow(), vec![(10, 10)]);
    }

    #[test]
    fn observe_calls_listener_on_change() {
        let store = Store::new(TestState {
            count: 5,
            label: String::new(),
            multiplier: 3,
        });

        let calls: Rc<RefCell<Vec<(i32, i32)>>> = Rc::new(RefCell::new(Vec::new()));
        let calls_clone = calls.clone();

        let _unsub = store.observe(
            |s| s.count * 2,
            move |new_val, old_val| {
                calls_clone.borrow_mut().push((*new_val, *old_val));
            },
        );

        store.set_state(TestState {
            count: 10,
            label: String::new(),
            multiplier: 3,
        });
        store.set_state(TestState {
            count: 7,
            label: String::new(),
            multiplier: 3,
        });

        assert_eq!(*calls.borrow(), vec![(10, 10), (20, 10), (14, 20)]);
    }

    #[test]
    fn observe_does_not_call_listener_when_selected_value_unchanged() {
        let store = Store::new(TestState {
            count: 5,
            label: String::new(),
            multiplier: 3,
        });

        let calls: Rc<RefCell<Vec<(i32, i32)>>> = Rc::new(RefCell::new(Vec::new()));
        let calls_clone = calls.clone();

        let _unsub = store.observe(
            |s| s.count * 2,
            move |new_val, old_val| {
                calls_clone.borrow_mut().push((*new_val, *old_val));
            },
        );

        // Changing multiplier should NOT trigger the observer (it only watches count*2)
        store.set_state(TestState {
            count: 5,
            label: String::new(),
            multiplier: 5,
        });

        // Only the initial call
        assert_eq!(*calls.borrow(), vec![(10, 10)]);
    }

    #[test]
    fn observe_unsubscribe_stops_notifications() {
        let store = Store::new(TestState {
            count: 5,
            label: String::new(),
            multiplier: 3,
        });

        let calls: Rc<RefCell<Vec<i32>>> = Rc::new(RefCell::new(Vec::new()));
        let calls_clone = calls.clone();

        let unsub = store.observe(
            |s| s.count * 2,
            move |new_val, _| {
                calls_clone.borrow_mut().push(*new_val);
            },
        );

        store.set_state(TestState {
            count: 10,
            label: String::new(),
            multiplier: 3,
        });
        assert_eq!(*calls.borrow(), vec![10, 20]);

        unsub();

        store.set_state(TestState {
            count: 15,
            label: String::new(),
            multiplier: 3,
        });
        // No new calls after unsubscribe
        assert_eq!(*calls.borrow(), vec![10, 20]);
    }

    #[test]
    fn multiple_observers() {
        let store = Store::new(TestState {
            count: 5,
            label: String::new(),
            multiplier: 3,
        });

        let calls1: Rc<RefCell<Vec<i32>>> = Rc::new(RefCell::new(Vec::new()));
        let calls2: Rc<RefCell<Vec<i32>>> = Rc::new(RefCell::new(Vec::new()));
        let c1 = calls1.clone();
        let c2 = calls2.clone();

        let _unsub1 = store.observe(
            |s| s.count * 2,
            move |new_val, _| {
                c1.borrow_mut().push(*new_val);
            },
        );
        let _unsub2 = store.observe(
            |s| s.count * 2,
            move |new_val, _| {
                c2.borrow_mut().push(*new_val);
            },
        );

        store.set_state(TestState {
            count: 10,
            label: String::new(),
            multiplier: 3,
        });

        assert_eq!(*calls1.borrow(), vec![10, 20]);
        assert_eq!(*calls2.borrow(), vec![10, 20]);
    }

    #[test]
    fn create_selector_composes() {
        let selector = create_selector(|s: &TestState| s.count, |count| count * 2);

        let state = TestState {
            count: 5,
            label: String::new(),
            multiplier: 1,
        };
        assert_eq!(selector(&state), 10);
    }

    #[test]
    fn update_fields_triggers_notification() {
        let store = Store::new(TestState {
            count: 0,
            label: "hello".to_string(),
            multiplier: 1,
        });

        let calls = Rc::new(Cell::new(0));
        let cc = calls.clone();
        let _unsub = store.subscribe(move |_| {
            cc.set(cc.get() + 1);
        });

        store.update_fields(|s| {
            s.count = 42;
        });

        assert_eq!(store.get_state().count, 42);
        assert_eq!(store.get_state().label, "hello"); // unchanged
        assert_eq!(calls.get(), 1);
    }
}
