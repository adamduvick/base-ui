# ADR 001: Reactive State via Leptos Reactive Stores

## Status

Accepted

## Context

In React Base UI, every component part re-renders when its parent re-renders. `className(state)` and `style(state)` callbacks are called on every render, receiving the current state. This is "free" reactivity -- React's top-down rendering model guarantees it.

In the Leptos port, component functions run **once**. Child components previously resolved `ClassProp`/`StyleProp` at mount into a static `String`/`Option<String>` and passed that to the view. If the root's state changed (e.g., progress value updates, fieldset becomes disabled), children's class/style/data-attributes **did not update**.

Avatar already handled this correctly -- its context stored `ReadSignal<ImageLoadingStatus>`, and children resolved class/style inside `move ||` closures that called `.get()`, creating signal subscriptions. Progress, Meter, and Fieldset stored plain values in context and resolved once.

### What React does

```
Parent re-renders -> children re-render -> className(state) called with fresh state -> DOM updates
```

Every child sees the latest state on every render. No explicit subscription needed.

### What Leptos requires

```
Signal changes -> closures that called .get() re-execute -> DOM updates
```

State must live in signals. Consumers must read signals inside reactive closures. Anything resolved outside a closure is static forever.

### Why reactive stores

Leptos's `reactive_stores` crate provides `#[derive(Store)]` which wraps a plain struct in a reactive store with **field-level granularity**. Each field gets its own reactive trigger -- reading `store.status().get()` inside a closure subscribes that closure to `status` changes only, not to changes in other fields.

This is the most idiomatic Leptos approach because:

- **Fine-grained**: Only closures that read a specific field re-execute when that field changes
- **Copy**: `Store<T>` is `Copy` -- can be freely passed into closures and provided via context
- **Natural API**: Callback props receive the store and call `.field().get()` to subscribe to exactly the fields they care about

## Decision

### 1. State structs become reactive stores

State structs get `#[derive(Store)]`. The store is created in the root component and provided via context:

```rust
use reactive_stores::Store;

#[derive(Store)]
pub struct ProgressState {
    pub status: ProgressStatus,
}

// In ProgressRoot:
let state_store = Store::new(ProgressState { status });
provide_context(state_store);
```

`Store<ProgressState>` is `Copy` -- it's a reactive graph handle, not the data itself.

### 2. ClassProp/StyleProp/RenderProp parameterized over `Store<T>` directly

The type parameter `S` of `ClassProp<S>` becomes `Store<StateType>`. Users see `Store<ProgressState>` directly -- no type aliases. The `ClassProp` definition itself does not change -- it's already generic over `S`. Only the usage sites change:

```rust
// Before
class: ClassProp<ProgressState>    // callback receives &ProgressState

// After
class: ClassProp<Store<ProgressState>>  // callback receives &Store<ProgressState>
```

Static impls (`From<&str>`, `From<String>`) still work -- they ignore the state parameter. Closure impls work -- the user's callback receives `&Store<ProgressState>` and calls `.field().get()` for fine-grained subscriptions:

```rust
// User writes:
<ProgressTrack class=|store: &Store<ProgressState>| {
    match store.status().get() {
        ProgressStatus::Complete => "complete".into(),
        _ => String::new(),
    }
} />

// Or with a static string (unchanged):
<ProgressTrack class="my-track" />
```

The generated `ProgressStateStoreFields` trait is automatically available in each component module so consumers can call field accessors.

### 3. State-driving props accept `Signal<T>`

Props that drive component state become `Signal<T>` with `#[prop(into)]`. This accepts both plain values and signals transparently:

| Component | Prop | Before | After |
|-----------|------|--------|-------|
| ProgressRoot | `value` | `Option<f64>` | `Signal<Option<f64>>` |
| MeterRoot | `value` | `f64` | `Signal<f64>` |
| FieldsetRoot | `disabled` | `Option<bool>` | `Signal<bool>` |

Configuration props that don't drive state (`min`, `max`, `orientation`, `id`) stay as plain values. Separator has no children or context -- no `Signal` needed.

Note: The plan originally specified `MaybeSignal<T>`, but `MaybeSignal` is deprecated in leptos 0.8 in favor of `Signal<T>`, which is `Copy` and has efficient `From<T>` implementations. We use `Signal<T>` throughout.

### 4. Root syncs reactive props into store via Effect

When a `Signal` prop changes, an `Effect` updates the corresponding store field:

```rust
// ProgressRoot
let state_store = Store::new(ProgressState {
    status: compute_status(value.get_untracked(), max),
});

Effect::new(move |_| {
    let new_status = compute_status(value.get(), max);
    state_store.status().set(new_status);
});
```

This creates the reactive chain:

1. `value` signal changes
2. Effect re-runs, computes new status, calls `state_store.status().set()`
3. Store field's trigger fires
4. Closures subscribed to `store.status()` re-execute (class/style/data-attrs update)

### 5. Children resolve class/style/data-attrs in reactive closures

Every child component reads from the store inside `move ||` closures:

```rust
let store = ctx.state;

view! {
    <div
        class=move || class.resolve_option(&store)
        style=move || style.resolve_option(&store)
        data-progressing=move || {
            if store.status().get() == ProgressStatus::Progressing { Some("") } else { None }
        }
    />
}
```

Inside `class.resolve_option(&store)`, the user's callback calls `store.status().get()`. This creates a subscription within the view's reactive scope. When `status` changes, only this closure re-executes -- not the entire component.

### 6. Context struct holds store + raw reactive values

The context provides both the store (for class/style/data-attrs) and raw reactive values for internal computation (indicator width, ARIA attributes, formatted display):

```rust
#[derive(Clone, Copy)]
struct ProgressContext {
    state: Store<ProgressState>,           // for class/style/data-attrs
    value: Signal<Option<f64>>,            // for indicator width, ARIA attrs
    min: f64,                               // static config
    max: f64,                               // static config
    formatted_value: Memo<String>,          // derived from value
    label_id: ReadSignal<Option<String>>,
    set_label_id: WriteSignal<Option<String>>,
}
```

All fields are `Copy` (`Store`, `Signal`, `Memo`, `ReadSignal`, `WriteSignal` are all `Copy`), so the context struct is `Copy`.

### 7. Avatar uses Store directly as context

Avatar is the simplest case -- the store is provided directly via `provide_context(store)` without a wrapper struct, since it only needs the loading status:

```rust
let store = Store::new(AvatarState {
    image_loading_status: ImageLoadingStatus::Idle,
});
provide_context(store);

// Children:
let store = expect_context::<Store<AvatarState>>();
store.image_loading_status().set(ImageLoadingStatus::Loaded);
```

## Constraints

- **`ClassProp<S>` / `StyleProp<S>` are not Clone or Copy** -- they contain `Box<dyn Fn>`. Each prop is moved into exactly one closure via the `if render.is_custom() { return ... }` / `else { view! { ... } }` pattern.

- **`Children` is `FnOnce`** -- called once at mount. Child *content* does not re-render when parent state changes. But child *components* (ProgressTrack, etc.) subscribe to context signals independently -- this is correct Leptos behavior.

- **`Store<T>` is `Copy`** -- it's a reactive graph handle. Can be freely captured in multiple closures.

- **`#[derive(Store)]` generates a `{Name}StoreFields` trait** -- must be in scope where field accessors are used. Since our state structs and components live in the same module, this is automatic. For external consumers, the generated trait is public and accessible from each component module.

- **`reactive_stores` 0.3** is compatible with leptos 0.8 via the shared `reactive_graph ^0.2` dependency.

## Consequences

### Benefits

- Matches React's behavioral contract -- class/style/data-attrs update when state changes
- **Fine-grained** -- only closures reading a specific field re-execute when that field changes (better than React's full re-render)
- API is backward-compatible at call sites -- callers passing plain values (`value=Some(50.0)`) still work via `Signal`'s `From` impl
- Enables new use cases -- callers can pass signals for animated progress bars, dynamic disable/enable
- Idiomatic Leptos -- uses the framework's reactive store primitive, not a custom solution
- Consistent pattern across all components (Separator, Progress, Meter, Fieldset, Avatar)

### Costs

- New dependency: `reactive_stores = "0.3"`
- Each root creates a Store + an Effect (negligible -- Leptos reactive graph is cheap)
- Prop type signatures change: `ClassProp<ProgressState>` to `ClassProp<Store<ProgressState>>`
- Callbacks that were `|state: &ProgressState| state.status` become `|store: &Store<ProgressState>| store.status().get()`

### Components affected

| Component | Changes |
|-----------|---------|
| Separator | `#[derive(Store)]` on state, `Store::new` in component, prop types updated |
| Progress | Full reactive: `Signal` value, `Effect` sync, `Memo` derived values, reactive closures in all 5 parts |
| Meter | Full reactive: `Signal` value, `Memo` derived values, reactive closures in all 5 parts |
| Fieldset | `Signal` disabled, `Effect` sync, reactive closures in both parts |
| Avatar | `Store` replaces signal pair in context, all 3 parts updated |
