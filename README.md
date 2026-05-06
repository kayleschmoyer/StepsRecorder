# StepForge Recorder

## Purpose

StepForge Recorder is a modern Windows desktop Steps Recorder app.

The user presses **Start Recording**, performs actions inside a Windows desktop application, and the app automatically records each meaningful mouse click as a documented step. Each step includes a screenshot of the visible monitor where the click occurred, step metadata, and editable text. After recording, the user can review, edit, reorder, delete, annotate, and export the steps into a polished Microsoft Word document or PDF.

## Product Vision

The app should be dramatically better than the legacy Windows Steps Recorder experience.

It should be:

- Simple enough for non-technical users.
- Powerful enough for QA, support, implementation, training, and documentation teams.
- Polished enough to generate documents that can be sent to customers, developers, managers, or vendors.
- Safe enough to help users avoid accidentally sharing sensitive screenshots.
- Fast enough to record normal Windows desktop workflows without slowing the user down.

## Primary Use Case

A user needs to document a workflow inside a Windows desktop application.

Example:

1. User opens StepForge Recorder.
2. User clicks **Start Recording**.
3. User performs actions inside another Windows desktop app.
4. Each mouse click becomes a numbered step.
5. Each step receives a screenshot.
6. User clicks **Stop Recording**.
7. User reviews and edits the captured steps.
8. User exports the final guide as `.docx` or `.pdf`.

## Target Users

- QA testers
- Implementation specialists
- Support teams
- Product analysts
- Business analysts
- Trainers
- Technical writers
- Internal software teams documenting desktop workflows

## Target Platform

Initial platform:

- Windows 11
- Windows 10 where technically feasible

Cross-platform support is not part of the MVP.

## Recommended Modern Tech Stack

### Desktop Shell

- Tauri 2
- Rust native backend
- Windows API integration through Rust

### Frontend

- React 19
- TypeScript
- Vite
- CSS Modules or vanilla CSS with design tokens
- No Tailwind unless explicitly approved

### State Management

- Zustand for client UI state

### Local Storage

- SQLite for session metadata, step records, and settings
- Local file system storage for screenshots and generated exports

### Document Generation

- DOCX export from structured step data
- PDF export from the same document model

### Testing

- Vitest for frontend unit tests
- React Testing Library for component behavior
- Playwright for end-to-end UI tests
- Rust unit/integration tests for native capture, file, export, and system logic

## Core Commands

These commands should exist once the project is initialized.

```bash
pnpm install
pnpm dev
pnpm test
pnpm lint
pnpm build
pnpm tauri dev
pnpm tauri build
```

## Important Project Rules

- The design system must be followed at all times.
- Do not create large monolithic files.
- Do not invent product requirements.
- Do not add libraries without a clear reason.
- Native Windows capture logic belongs in the Rust/Tauri layer.
- UI-only logic belongs in the React layer.
- Screenshots must be stored locally and referenced by step records.
- The user must be able to edit steps after recording.
- The app must support DOCX and PDF export.
- The app must remain simple and clean for non-technical users.

## Documentation Order for AI Agents

Before writing code, read these files in this order:

1. `README.md`
2. `docs/AI_AGENT_RULES.md`
3. `docs/PRODUCT_REQUIREMENTS.md`
4. `docs/ARCHITECTURE.md`
5. `docs/DESIGN_SYSTEM.md`
6. `docs/CODING_STANDARDS.md`
7. `docs/DATABASE_SCHEMA.md`
8. `docs/API_CONTRACT.md`
9. `docs/TESTING_STRATEGY.md`
10. `docs/SECURITY_REQUIREMENTS.md`
11. `docs/ERROR_HANDLING.md`
12. `docs/PERFORMANCE_REQUIREMENTS.md`
13. `docs/ACCESSIBILITY.md`
14. `docs/DEPLOYMENT.md`
15. `docs/ROADMAP.md`

## Current Development Commands

```bash
npm install
npm run dev
npm run build
npm run tauri:dev
```
