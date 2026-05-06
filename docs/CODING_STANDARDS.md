# Coding Standards

## Core Principle

Code must be clean, readable, maintainable, and production-grade.

The app should be built like a real product, not a prototype full of shortcuts.

## General Rules

- Use TypeScript for all frontend code.
- Use Rust for native/Tauri logic.
- Prefer small files over large files.
- Prefer focused functions over large procedural blocks.
- Use descriptive names.
- Avoid clever code.
- Avoid duplicated logic.
- Avoid magic values.
- Keep business rules outside visual components.
- Keep native capture logic outside React.
- Add comments only when the reason is not obvious.

## File Size Guidance

These are guidelines, not hard limits:

- React component: under 250 lines
- Store file: under 200 lines
- Service/client file: under 250 lines
- Rust module: under 300 lines
- Utility file: under 150 lines

If a file grows beyond these, consider splitting it.

## TypeScript Rules

- Avoid `any`.
- Use explicit types for command inputs and outputs.
- Use discriminated unions for complex state.
- Use `unknown` for unknown errors and narrow them safely.
- Keep shared app types in `/src/types`.
- Keep feature-specific types inside the feature folder.

## React Rules

- Use functional components.
- Keep components focused.
- Extract repeated UI patterns into reusable components.
- Avoid deeply nested JSX.
- Avoid excessive prop drilling.
- Use stores for cross-feature UI state.
- Use local component state for isolated UI behavior.
- Do not put Tauri `invoke` calls directly inside many components.
- Use typed API wrappers.

## CSS Rules

- Use design tokens from `DESIGN_SYSTEM.md`.
- Do not hardcode colors in components.
- Do not introduce random spacing values.
- Keep CSS scoped by component or feature.
- Avoid global CSS except tokens, reset, and app shell primitives.
- Do not use Tailwind unless explicitly approved.

## Rust Rules

- Keep Tauri commands thin.
- Put business/native logic in dedicated modules.
- Use typed structs for command input/output.
- Use `Result<T, AppError>` patterns.
- Map internal errors to user-safe error responses.
- Do not panic for recoverable errors.
- Avoid global mutable state unless wrapped safely.
- Write tests for capture-adjacent logic where feasible.

## Error Handling

Every async operation must handle errors.

Frontend errors should show:

- Friendly title
- Short explanation
- Suggested next action

Backend errors should include:

- Stable error code
- Safe user message
- Technical detail for logs where appropriate

## Logging

Logs should help diagnose issues without leaking private content.

Do not log:

- Screenshot image contents
- Passwords
- Tokens
- Clipboard data
- Full sensitive paths unless needed
- User-entered descriptions unless necessary

## Naming Conventions

### Frontend

- Components: `PascalCase`
- Hooks: `useThing`
- Stores: `thingStore`
- Types: `Thing`, `ThingInput`, `ThingResult`
- API wrappers: `thingApi`
- CSS modules: `ComponentName.module.css`

### Rust

- Modules: `snake_case`
- Structs/enums: `PascalCase`
- Functions: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`

## Validation Rules

Validate all external input.

This includes:

- Tauri command input
- File paths
- Export options
- Settings values
- Session IDs
- Step IDs

## Dependency Rules

Before adding a dependency, document why it is needed.

A dependency is acceptable when it:

- Avoids risky custom native code.
- Is actively maintained.
- Has a clear purpose.
- Does not bloat the app unnecessarily.
- Does not conflict with the architecture.

## Refactoring Rules

Refactor only when it improves:

- Maintainability
- Safety
- Testability
- Performance
- Consistency with docs

Do not rewrite working systems for style alone.

## Documentation Updates

When behavior changes, update the relevant `.md` file.

Examples:

- New setting: update `PRODUCT_REQUIREMENTS.md` and `DATABASE_SCHEMA.md`.
- New command: update `API_CONTRACT.md`.
- New export option: update `PRODUCT_REQUIREMENTS.md` and `API_CONTRACT.md`.
- New UI pattern: update `DESIGN_SYSTEM.md`.
