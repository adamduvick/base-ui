# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

Base UI is a headless, unstyled React component library. It's a monorepo managed with pnpm workspaces and Lerna.

## Common Commands

```bash
# Install dependencies
pnpm install

# Build packages
pnpm build

# Type checking
pnpm typescript

# Linting & formatting
pnpm eslint
pnpm stylelint
pnpm markdownlint
pnpm prettier

# Run tests (pick one environment)
pnpm test:jsdom NumberField --no-watch      # Fast, JSDOM (no layout)
pnpm test:chromium NumberField --no-watch   # Real Chromium browser

# Regenerate API docs after changing public props/JSDoc
pnpm docs:api

# Update error codes after adding/modifying error messages
pnpm extract-error-codes

# Run the docs site locally (port 3005)
pnpm docs:dev
```

## Monorepo Structure

- `packages/react/` — `@base-ui/react`: the main component library
- `packages/utils/` — `@base-ui/utils`: shared React utilities (hooks, store, formatters)
- `docs/` — Next.js documentation site
- `docs/src/app/(docs)/react/` — public documentation pages
- `docs/src/app/(private)/experiments/` — dev experiments for manual browser testing
- `test/` — shared vitest setup, e2e tests, regression tests, bundle size checks
- `playground/` — Vite playground app

## Component Architecture

### File Layout

Each component lives in `packages/react/src/<component-name>/` with sub-directories per part:

```
popover/
  root/         PopoverRoot.tsx, PopoverRootContext.ts
  trigger/      PopoverTrigger.tsx
  popup/        PopoverPopup.tsx
  positioner/   PopoverPositioner.tsx
  portal/       PopoverPortal.tsx
  ...
  store/        PopoverStore.ts, PopoverHandle.ts
  index.ts      → exports namespace: export * as Popover from './index.parts'
  index.parts.ts → maps internal names to short names (Root, Trigger, Popup, etc.)
```

Consumers import as `import { Popover } from '@base-ui/react/popover'` and use `<Popover.Root>`, `<Popover.Trigger>`, etc.

### Component Internals Pattern

Every component part follows this structure:

1. `'use client'` directive at the top
2. Props typed with `BaseUIComponentProps<'element', State>` — adds `className` (string or function of state), `render` prop, and `style` (object or function of state)
3. State is a plain object typed as `ComponentName.State`
4. Rendering uses **`useRenderElement()`** — handles the `render` prop, merges refs, applies `data-*` attributes from state, and merges class names
5. Types exported as `ComponentName.Props` and `ComponentName.State` via a namespace

### Context Pattern

Compound components share state via React context. Each has a typed context with a `use<Component>RootContext()` hook that throws a formatted error if missing (unless `optional: true`).

### Store Pattern (ReactStore)

Complex components use `ReactStore` from `@base-ui/utils/store` — a pub/sub store with React integration (`useState`, `useSyncedValues`, `useControlledProp`, etc.). The store's `Context` object holds non-reactive refs and callbacks.

### Data Attributes

Each part has a `ComponentNameDataAttributes` file defining `data-*` attributes (e.g., `data-open`, `data-disabled`). These are applied automatically by `useRenderElement` based on the component's state.

## Testing

### Setup

- Tests are co-located with source: `PopoverRoot.tsx` → `PopoverRoot.test.tsx`
- `.spec.tsx` files are excluded from the test runner
- `const { render, clock } = createRenderer()` at the describe level
- Import test utilities from `#test-utils` (alias to `packages/react/test/index.ts`)
- `BASE_UI_ANIMATIONS_DISABLED = true` is set globally; animations are off by default in tests

### Key Patterns

- `describeConformance()` — standard test suite checking prop forwarding, ref forwarding, `render` prop, className
- `popupConformanceTests()` — shared tests for popup components
- `isJSDOM` — use with `it.skipIf(isJSDOM)` / `describe.skipIf(isJSDOM)` for tests requiring real browser layout
- Use vitest's `expect()` and `fn()` for all new tests (the repo is transitioning away from chai/sinon)
- Use `screen`, `fireEvent`, `waitFor` from `@mui/internal-test-utils`

## Code Guidelines

@AGENTS.md
