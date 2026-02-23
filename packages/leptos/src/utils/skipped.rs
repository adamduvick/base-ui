//! Documentation of skipped utils units (not ported to Leptos).
//!
//! These React utils have no meaningful Leptos equivalent per porting rule R6
//! (skip React-specific internals) or R3 (map concepts, don't transliterate).
//!
//! ## `utils/reactVersion`
//! React version detection. No equivalent needed — there's no version-gating for Leptos.
//!
//! ## `utils/safeReact`
//! Workaround for a React-specific spread issue (material-ui#41190). Not applicable.
//!
//! ## `utils/testUtils`
//! TypeScript type-level test utility (`IfEquals`, `expectType`) and JSDOM detection.
//! TypeScript type machinery has no Rust equivalent; Rust's type system enforces
//! correctness at compile time. JSDOM detection is not applicable.
//!
//! ## `utils/fastObjectShallowCompare`
//! Shallow comparison of JS objects. In Rust, use `PartialEq` derive on structs.
//!
//! ## `utils/mergeObjects`
//! Merge two optional JS objects via spread. In Rust, use struct update syntax
//! (`Foo { field, ..default }`) or `Option::or` / `Option::unwrap_or_else`.
//!
//! ## `utils/useForcedRerendering`
//! Forces a React re-render by setting dummy state. Not needed in Leptos — signals
//! trigger granular updates automatically.
//!
//! ## `utils/useIsoLayoutEffect`
//! Isomorphic `useLayoutEffect` that becomes a no-op on the server. In Leptos CSR mode,
//! `create_render_effect` is always available. No wrapper needed.
//!
//! ## `utils/useOnFirstRender`
//! Runs a function on the first render only. In Leptos, code in the component body
//! before the `view!` macro runs exactly once. No hook wrapper needed.
//!
//! ## `utils/useRefWithInit`
//! Lazy-initialized `useRef`. In Leptos, local variables in the component body
//! serve this purpose, or use `StoredValue` for heap-allocated data.
//!
//! # Tier 1
//!
//! ## `utils/fastHooks`
//! React-specific render-phase hook optimization framework (`fastComponent`,
//! `fastComponentRef`). Uses `forwardRef`, `useRefWithInit`, and a global instance
//! tracking system tied to React's render cycle. No Leptos equivalent — Leptos
//! components don't have a render-phase/commit-phase distinction.
//!
//! ## `utils/getReactElementRef`
//! Extracts the `ref` from a React element, handling React 18 vs 19 differences.
//! Not applicable — Leptos uses `NodeRef` directly, not element-level ref extraction.
//!
//! ## `utils/inertValue`
//! React version compatibility shim for the `inert` HTML attribute (returns boolean
//! in React 19+, string in older versions). Not needed — Leptos handles boolean
//! attributes natively.
//!
//! ## `utils/useMergedRefs`
//! Merges multiple React refs (callback refs, ref objects) into a single ref.
//! Not needed — Leptos uses `NodeRef` which is a single reference type. If
//! multiple components need access to the same node, share the `NodeRef`.
//!
//! ## `utils/useStableCallback`
//! Stabilizes a callback's identity across React renders using `useInsertionEffect`.
//! Prevents unnecessary effect re-runs when a callback's captured values change.
//! Not needed — Leptos closures captured in the component body are created once
//! and their identity is inherently stable.
//!
//! ## `utils/useValueAsRef`
//! Turns a reactive value into a non-reactive ref to avoid effect re-triggers.
//! Not needed — Leptos's fine-grained reactivity model lets you choose exactly
//! which signals to track. Use `.get_untracked()` to read without subscribing.
