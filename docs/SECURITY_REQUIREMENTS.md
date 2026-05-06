# Security Requirements

## Security Goal

The app must safely record desktop workflow documentation without intentionally capturing or exposing sensitive information.

Because the app captures screenshots, privacy and local data handling are critical.

## Local-First Security Model

MVP is local-first.

The app does not require:

- Login
- Cloud sync
- Remote storage
- External API calls
- Team accounts

All sessions, screenshots, and exports are stored locally.

## Sensitive Data Risk

Screenshots may accidentally contain:

- Customer information
- Personal information
- Account numbers
- Emails
- Internal business data
- Prices
- Vendor data
- Password fields
- Tokens
- Desktop notifications

The app must remind users to review screenshots before export.

## Data Storage Rules

- Store screenshots locally.
- Store exports locally.
- Store SQLite database locally.
- Do not upload files automatically.
- Do not send telemetry without explicit approval.
- Do not store screenshot images as database blobs.
- Do not store secrets in source code.

## Recording Restrictions

The MVP must not intentionally record:

- Keystrokes
- Passwords
- Clipboard contents
- Audio
- Video
- Hidden windows
- Background windows

The app records mouse click events only for the purpose of creating documented steps.

## Screenshot Privacy

Required:

- Show privacy reminder before export.
- Provide redaction tools.
- Allow screenshot deletion through step deletion.
- Allow users to review every screenshot before export.

Recommended:

- Add a visible warning near export:
  `Review screenshots before exporting. Screenshots may include sensitive information.`

## File Access

The app should only write to:

- Its application data folder.
- User-selected export locations.
- User-selected file paths.

Do not write to arbitrary system folders unless the user explicitly chooses them.

## Error Logging

Logs must not include:

- Screenshot contents
- Passwords
- Tokens
- Full document text unless needed
- Sensitive clipboard data
- Keystrokes

Logs may include:

- Error codes
- Operation names
- Safe file operation failures
- Export failure reason
- Timestamp
- App version

## Dependency Security

Before adding dependencies, consider:

- Is it maintained?
- Is it necessary?
- Does it touch native permissions?
- Does it increase attack surface?
- Does it introduce network access?
- Does it process untrusted files?

## Tauri Security

Tauri configuration should follow least privilege.

Rules:

- Only enable required Tauri capabilities.
- Do not enable broad file system access unnecessarily.
- Do not expose unnecessary commands.
- Validate all command input.
- Restrict file dialogs to expected use cases where possible.

## Export Safety

Before export:

- Validate session exists.
- Validate active steps exist.
- Validate screenshot files exist.
- Confirm privacy reminder if enabled.
- Use user-selected output path.
- Avoid overwriting files without confirmation.

## Future Security Considerations

Future versions may need:

- Encryption at rest
- Workspace/team permissions
- Secure cloud storage
- Audit logs
- Enterprise policy controls
- Automatic sensitive data detection
- Admin-controlled redaction rules
