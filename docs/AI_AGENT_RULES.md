# AI Agent Rules

## Prime Directive

Build this app carefully, incrementally, and without inventing requirements.

This is a Windows desktop Steps Recorder app. It must record user clicks, capture screenshots, allow review/editing, and export polished Microsoft Word and PDF documentation.

## Required Reading Before Coding

Before writing code, modifying code, adding dependencies, or changing architecture, read:

1. `README.md`
2. `docs/PRODUCT_REQUIREMENTS.md`
3. `docs/ARCHITECTURE.md`
4. `docs/DESIGN_SYSTEM.md`
5. `docs/CODING_STANDARDS.md`
6. `docs/DATABASE_SCHEMA.md`
7. `docs/API_CONTRACT.md`
8. `docs/TESTING_STRATEGY.md`
9. `docs/SECURITY_REQUIREMENTS.md`

## Required First Response Before Coding

After reading the documentation, summarize:

1. The app goal
2. The MVP scope
3. The architecture rules
4. The data model
5. The first safe implementation step

Do not write code until this summary is complete.

## Development Behavior

When making changes:

- Make the smallest safe change.
- Prefer clear, maintainable code over clever code.
- Keep files focused.
- Follow the documented folder structure.
- Reuse existing components and services before creating new ones.
- Do not duplicate logic.
- Do not silently change requirements.
- Do not silently change architecture.
- Do not create large files.
- Do not mix UI code with native capture logic.
- Do not mix business rules with visual components.

## Design Enforcement

The `DESIGN_SYSTEM.md` file is mandatory.

Every screen, component, modal, button, form, state, and export preview must follow the design system.

If a UI decision is not covered, use the closest existing pattern and document the decision.

## Native Capture Rules

The app records Windows desktop workflows.

Capture, input hooks, screen/monitor detection, privacy filtering, file access, and export generation are system-level concerns handled through the Rust/Tauri layer.

Frontend React components must not directly perform native capture.

## Recording Rules

During recording:

- A click should create a new step.
- A screenshot should be captured as close as possible to the click event.
- The app should avoid duplicate accidental click spam where possible.
- The app should store enough metadata to make each step editable later.
- Recording should not block or noticeably slow normal user work.

## Screenshot Rules

The screenshot should represent what the user sees on the relevant screen at the time of the click.

MVP assumption:

- Capture the visible display/monitor where the click occurred.
- Do not attempt to capture hidden windows.
- Do not attempt to capture windows behind other windows.
- Do not attempt to capture windows in front of the target application separately.
- Capture the visible screen state at the moment of interaction.

Future versions may support active-window-only capture or target-application-only capture.

## Export Rules

Exports must be generated from the structured recording model, not from ad hoc UI screenshots.

Supported MVP exports:

- Microsoft Word `.docx`
- PDF `.pdf`

Generated documents must include:

- Title
- Optional description
- Recording date/time
- Step count
- Numbered steps
- Step title
- Step description
- Screenshot per step
- Optional annotations

## When Unsure

Do not guess.

Ask a concise question with clear choices, for example:

```text
I need one decision before coding:
A) Capture the whole monitor where the click happened
B) Capture only the active window
C) Support both as a setting
```

If the task can proceed safely with a documented MVP assumption, proceed and explicitly state the assumption.

## Output Format After Code Changes

Whenever code is changed, respond with:

1. Summary
2. Files changed
3. Why the change was made
4. How to test
5. Any known limitations

## Forbidden Behavior

Do not:

- Invent fields that are not in `DATABASE_SCHEMA.md`.
- Invent APIs not in `API_CONTRACT.md`.
- Add dependencies without a reason.
- Use Tailwind unless explicitly approved.
- Create one giant component for the app.
- Store screenshots in the database as blobs unless explicitly approved.
- Store secrets in source code.
- Log sensitive user content.
- Implement cloud sync in MVP.
- Add authentication in MVP unless requirements change.
- Record keystrokes in MVP.
- Capture passwords or masked fields intentionally.
- Export unreviewed sensitive screenshots without user confirmation.
