# Accessibility

## Accessibility Goal

The app should be usable by people who rely on keyboard navigation, clear contrast, readable text, and predictable UI behavior.

## General Requirements

- All buttons must be keyboard reachable.
- All form fields must have labels.
- Focus states must be visible.
- Dialogs must trap focus while open.
- Escape should close non-destructive dialogs.
- Destructive actions must require confirmation.
- Recording state must not rely only on color.
- Icons must have accessible labels where needed.

## Contrast

Text must have strong contrast against backgrounds.

Avoid low-contrast gray text for important actions.

## Keyboard Support

Required keyboard support:

- Start Recording button focusable.
- Stop Recording button focusable.
- Step list navigable.
- Edit fields usable with keyboard.
- Export dialog usable with keyboard.
- Screenshot editor toolbar usable with keyboard.

Suggested shortcuts:

- `Ctrl + N`: New recording
- `Ctrl + E`: Export
- `Delete`: Delete selected step after confirmation
- `Ctrl + S`: Save current edits
- `Esc`: Close dialog/editor where safe

## Screen Reader Labels

Interactive icon-only buttons must have labels.

Examples:

```text
Delete step
Edit screenshot
Move step up
Move step down
Export as PDF
```

## Recording Indicator

Recording state should include:

- Color
- Text label
- Optional pulsing icon

Example:

```text
Recording active
00:01:23
12 steps captured
```

## Error Messages

Error messages should be:

- Clear
- Specific
- Near the relevant control where possible
- Announced appropriately if using ARIA live regions

## Motion Sensitivity

Animations should be subtle.

Avoid excessive flashing or motion.

Recording pulse should be gentle.

## Screenshot Editor Accessibility

The screenshot editor is visual by nature, but baseline accessibility still matters.

Required:

- Toolbar buttons must be keyboard reachable.
- Tool names must be visible or accessible.
- Save/cancel must be clear.
- Redaction/crop operations should expose numeric fields in future versions if possible.

## Exported Document Accessibility

Generated documents should:

- Use real headings where possible.
- Use readable text sizes.
- Avoid tiny muted text.
- Keep step numbering clear.
- Include step descriptions as text, not only screenshots.
