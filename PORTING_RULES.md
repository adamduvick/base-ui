# Porting Rules: Base UI React → Leptos

Rules for porting Base UI components from React to Leptos (Rust/WASM). Every ported unit must be validated before moving on.

## Workflow

### 1. Pick a unit

Run `python analyze_deps.py --ready` and pick a unit marked `[READY]`. All of its dependencies must already be `done`. Work tier-by-tier, lowest first. Units within the same tier can be worked in parallel.

### 2. Mark it in progress

```bash
python analyze_deps.py --set-status <unit> in_progress
```

### 3. Port it

Follow the rules below.

### 4. Validate it

Every unit must pass validation (see validation rules) before marking done.

### 5. Mark it done

```bash
python analyze_deps.py --set-status <unit> done
```

Re-run `python analyze_deps.py --ready` to see what's newly unblocked.

---

## Porting Rules

### R1: One unit at a time

Do not start porting a unit until all its dependencies are `done`. The dependency graph in `analyze_deps.py` is the source of truth. If a unit has unported dependencies, it is not ready.

### R2: Read the React source first

Before writing any Rust, read every source file in the unit. Understand:
- What public API it exposes (props, hooks, context)
- What internal state it manages
- What side effects it has (DOM manipulation, event listeners, timers)
- What data attributes it applies
- What accessibility semantics it provides (ARIA roles, attributes, keyboard handling)

### R3: Map concepts, don't transliterate

React and Leptos have different idioms. Map concepts to their Leptos equivalents:

| React concept | Leptos equivalent |
|---|---|
| `useState` / `useReducer` | `create_signal` / `create_rw_signal` |
| `useEffect` | `create_effect` |
| `useLayoutEffect` | `create_render_effect` |
| `useRef` | `create_node_ref` / `store_value` |
| `useCallback` / `useMemo` | `create_memo` / closures (often unnecessary) |
| `React.createContext` + `useContext` | `provide_context` / `use_context` |
| `forwardRef` | `NodeRef` prop |
| `render` prop / `children` as function | Render callbacks / `children: Children` |
| `className` as function of state | `class` with derived signal |
| `data-*` attributes from state | Derived attributes via `attr:` |
| ReactStore (pub/sub) | `RwSignal` + `create_effect` / `leptos_reactive` stores |
| Compound components via context | `provide_context` in parent, `use_context` in children |

### R4: Preserve the public API contract

The ported component must expose the same logical API:
- Same component parts (Root, Trigger, Popup, etc.)
- Same props (translated to Rust types and naming conventions)
- Same data attributes (e.g., `data-open`, `data-disabled`)
- Same ARIA roles, states, and keyboard interactions
- Same event callbacks (translated to Leptos event patterns)

Internal implementation details (hook names, store shape) do not need to match.

### R5: Preserve accessibility

Accessibility is non-negotiable. The ported component must:
- Apply the same ARIA roles and attributes
- Support the same keyboard interactions
- Manage focus the same way
- Have the same screen reader behavior

Test accessibility explicitly (see V3 below).

### R6: Skip React-specific internals

Do not port:
- `useRenderElement` — this is React's render-prop/className-function plumbing; Leptos handles rendering differently
- `describeConformance` / conformance test infrastructure — write idiomatic Leptos tests instead
- `BaseUIComponentProps` type machinery — use idiomatic Leptos component props
- `'use client'` directives — not applicable
- React-specific ref forwarding — use `NodeRef`

### R7: Handle external dependencies

| React dependency | Leptos approach |
|---|---|
| `@floating-ui/react` | `floating-ui` via wasm-bindgen bindings, or a Rust positioning library |
| `react-dom` | Direct DOM access via `web_sys` |
| `@mui/internal-test-utils` | Custom test harness (see validation) |

Document any external dependency decisions in the unit's module-level doc comment.

### R8: Use Rust idioms

- Use `enum` for state variants, not string unions
- Use `Option<T>` instead of nullable types
- Use the type system to enforce invariants at compile time where React enforced them at runtime
- Use `Result<T, E>` for fallible operations instead of throwing
- Prefer strong typing over `any` / dynamic dispatch

---

## Validation Rules

Every unit must pass validation before marking `done`. The type of validation depends on the unit category.

### V1: It compiles

`cargo check` must pass with zero errors. This is the minimum bar for every unit.

### V2: Unit tests for logic units

Units that are pure logic (`utils/*`, store, state machines, parsers, formatters) must have Rust unit tests covering:

- **Core behavior**: the happy path works as documented
- **Edge cases**: empty inputs, boundary values, overflow
- **Error paths**: invalid inputs produce correct errors
- **Equivalence**: for each significant test case in the React source's `.test.tsx`, write an equivalent Rust `#[test]`

Use this structure:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_behavior() { /* ... */ }

    #[test]
    fn test_edge_case() { /* ... */ }
}
```

Minimum: one test per public function/method. Aim for parity with the React test count.

### V3: Integration tests for component units

Component units (`react/*`) must have browser-based integration tests that verify:

- **Rendering**: the component mounts and produces the expected DOM structure
- **Data attributes**: correct `data-*` attributes appear in each state
- **ARIA**: correct roles, `aria-*` attributes, and labeling
- **Keyboard interaction**: Tab, Enter, Space, Escape, Arrow keys behave correctly
- **Mouse interaction**: click, hover, focus/blur behave correctly
- **State transitions**: open/close, check/uncheck, enable/disable work correctly
- **Callbacks**: event callbacks fire with the correct arguments

Use `wasm-bindgen-test` with `wasm_bindgen_test_configure!(run_in_browser)` for tests that need a real DOM. For headless CI, use `wasm-pack test --headless --chrome`.

### V4: Cross-reference React tests

For every unit, review the corresponding React `.test.tsx` file(s). For each test case:
1. **Port it** if it tests behavior that applies to the Leptos version
2. **Skip it** if it tests React-specific plumbing (render props, ref forwarding, className functions)
3. **Document** skipped tests with a comment explaining why

Track coverage by listing ported vs. skipped test cases in a comment block at the top of the Rust test module:
```rust
// Ported from SwitchRoot.test.tsx:
// ✓ toggles when clicked
// ✓ respects disabled prop
// ✓ fires onCheckedChange
// ✗ className function (React-specific)
// ✗ render prop (React-specific)
```

### V5: No regressions in done units

After porting a new unit, re-run tests for all `done` units to confirm no regressions. A new unit must not break previously ported units.

```bash
cargo test
```

### V6: Doc comments

Every public item (struct, enum, function, trait, component) must have a `///` doc comment explaining:
- What it does
- How it maps to the React original
- Any behavioral differences from the React version

---

## Cycle Handling

Some react units form dependency cycles (mostly through `react/utils`). For these:

1. Identify the minimal set of items needed from each dependency (often just a type or a single function)
2. Port the shared types/traits first as a standalone item
3. Use Rust's module system to break the cycle — define shared traits in a parent module, implement in children
4. Both sides of the cycle can be marked `in_progress` simultaneously but neither is `done` until both compile and pass tests together
5. Mark them `done` together in one batch

---

## Checklist per Unit

Before marking a unit `done`, confirm:

- [ ] All dependencies are `done`
- [ ] `cargo check` passes
- [ ] Unit tests pass (logic units) or integration tests pass (component units)
- [ ] React `.test.tsx` cases are cross-referenced and ported/skipped with justification
- [ ] Public API matches the React version's contract (props, events, ARIA, data attributes)
- [ ] Doc comments on all public items
- [ ] `cargo test` passes for all previously done units (no regressions)
- [ ] Status updated: `python analyze_deps.py --set-status <unit> done`
