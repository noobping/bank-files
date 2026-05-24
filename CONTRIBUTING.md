# Contributing

This is a Rust GTK/libadwaita project.

Contributions must be DRY, SRP, KISS, idiomatic Rust, GNOME-friendly, and Flathub-friendly.

## Required checks

Run before opening a pull request:

```bash
cargo fmt
cargo clippy
cargo test
```

## Code rules

- Prefer simple, readable Rust.
- Prefer immutable values.
- Prefer borrowing over cloning.
- Use `Result` and `Option`.
- Avoid `unwrap()` and `expect()` in production code.
- Keep functions and modules focused.
- Avoid global or shared mutable state.
- Use data pipelines and message passing.
- Keep blocking work off the GTK main thread.
- Use workers, async tasks, channels, or threads for IO, API calls, parsing, long-running work, and heavy UI preparation.
- Only final GTK widget creation, final UI updates, and UI event handling should run on the main thread.
- If UI work is heavy or renders a lot of data, prepare it in the background and show a loading placeholder until it is ready.

## Architecture and separation of concerns

Code must have a clear separation of concerns.

Separate these responsibilities:

- UI/widgets: GTK/libadwaita construction and visual state only.
- Actions/controllers: translate user actions into commands.
- State/models: typed application data and state.
- Services: IO, APIs, persistence, and external systems.
- Workers: background work, threading, and pipelines.
- Dialogs: shared dialog builder and dialog-specific configuration only.

Do not mix UI code with IO, business logic, parsing, persistence, or threading.

Apply patterns where they make code simpler and less repetitive:

- Builder pattern for dialogs and repeated UI construction.
- Command/message pattern for UI-to-worker communication.
- Pipeline pattern for validation, processing, and result delivery.
- Service pattern for IO, APIs, persistence, and external integrations.
- Factory/helper constructors only when they reduce duplication.

Do not use patterns for decoration. Use them to enforce DRY, SRP, KISS, and idiomatic Rust.

## File and module rules

Rust files over 300 lines are too large and must be split.

The only exception is when splitting the file would make the code less DRY, less SRP, less KISS, or less idiomatic Rust.

Do not split code just to create more files. Too many tiny files can also be bad.

Merge files when it reduces boilerplate, removes needless indirection, or makes the code easier to understand, as long as the result still follows DRY, SRP, KISS, idiomatic Rust, and the 300-line limit.

Group related files in folders with `mod.rs`.

Do not create folders for only one file, such as `pie/test.rs`. A folder must group multiple related files or have a clear reason to exist.

Example:

```text
src/
  dialogs/
    mod.rs
    builder.rs
    confirmation.rs
  workers/
    mod.rs
    pipeline.rs
    messages.rs
```

## Dialogs

All dialogs must use one shared dialog builder.

Do not duplicate dialog setup code.

The shared builder should handle titles, body text, buttons, suggested actions, destructive actions, cancellation, parent windows, defaults, keyboard behavior, and accessibility labels.

## GTK/libadwaita UI

- Use default GTK/libadwaita components unless it is absolutely impossible to meet the requirement otherwise.
- Follow GNOME Human Interface Guidelines.
- Keep Flathub packaging and sandboxing in mind.
- Do not create custom widgets, custom styling, or custom behavior when GTK/libadwaita already supports it.
- Custom UI must be justified in the pull request.
- Keep UI accessible and keyboard-friendly.

## Buttons

- Only actions that really add or modify data can and must use suggested styling.
- There may be only one visible suggested button per screen, page, dialog, or popup.
- Dangerous actions must use destructive/danger styling.
- Cancel actions must stay neutral.
- Navigation, view, filter, search, refresh, and menu actions must not be suggested.
- Buttons in headers, toolbars, and footers must be flat icon buttons.
- Only the one suggested button may also have text in a header.
- If more than two buttons are next to each other, use grouped buttons.
- If there are more than three grouped buttons, move them into a real dotted menu.
- Keep only the most important button or buttons outside the menu.
- Use a real menu, not a popover, for overflow actions.

## Dependencies

Do not add dependencies unless needed.

Prefer the Rust standard library, GTK, libadwaita, and existing project code first.

New dependencies must be maintained, license-compatible, and reasonable for Flathub.

## Tests

Add as many useful tests as possible.

Test every nook and cranny:

- Normal behavior.
- Edge cases.
- Error cases.
- Empty input.
- Invalid input.
- Boundary values.
- Data pipeline steps.
- Services and parsing logic.
- Dialog/action decision logic where practical.
- Regression cases for fixed bugs.

Prefer small focused tests over large fragile tests.

Do not skip tests just because code is internal. If behavior matters, test it.

## Pull request checklist

- [ ] Code is DRY, SRP, KISS, and idiomatic Rust.
- [ ] GNOME, GTK, libadwaita, and Flathub expectations are followed.
- [ ] Default GTK/libadwaita components are used unless custom UI is absolutely unavoidable.
- [ ] Any custom UI is clearly justified.
- [ ] Files stay under 300 lines unless clearly justified.
- [ ] Related files are grouped with `mod.rs`.
- [ ] No folder exists only to hold one file unless clearly justified.
- [ ] Tiny or over-fragmented files are merged when that makes the code simpler.
- [ ] Dialogs use the shared dialog builder.
- [ ] Buttons follow suggested/destructive/header/menu rules.
- [ ] Useful tests cover normal behavior, edge cases, error cases, and regressions.
- [ ] Blocking work is off the main thread.
- [ ] `cargo fmt`, `cargo clippy`, and `cargo test` pass.
