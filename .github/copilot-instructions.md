# GitHub Copilot Instructions

This is a Rust GTK/libadwaita project.

Generated code must be DRY, SRP, KISS, idiomatic Rust, GNOME-friendly, and Flathub-friendly.

## Rust

- Prefer simple, readable Rust.
- Prefer immutable values.
- Prefer borrowing over cloning.
- Use `Result`, `Option`, and pattern matching.
- Avoid `unwrap()` and `expect()` in production code.
- Avoid global or shared mutable state.
- Keep functions and modules focused.
- Use strong types instead of magic values.

## Architecture and separation of concerns

Generated code must separate responsibilities clearly.

Keep these concerns separate:

- UI/widgets: GTK/libadwaita construction and visual state only.
- Actions/controllers: convert user actions into commands.
- State/models: typed application data and state.
- Services: IO, APIs, persistence, and external systems.
- Workers: background work, threading, and pipelines.
- Dialogs: shared dialog builder and dialog-specific configuration only.

Never mix UI code with IO, business logic, parsing, persistence, or threading.

Apply patterns when they reduce duplication or enforce structure:

- Builder pattern for dialogs and repeated UI construction.
- Command/message pattern for UI-to-worker communication.
- Pipeline pattern for validation, processing, and result delivery.
- Service pattern for IO, APIs, persistence, and external integrations.
- Factory/helper constructors only when they reduce duplication.

Do not add patterns for decoration. Patterns must support DRY, SRP, KISS, and idiomatic Rust.

## Files and modules

Rust files over 300 lines are too large and must be split.

The only exception is when splitting would make the code less DRY, less SRP, less KISS, or less idiomatic Rust.

Group related files in folders with `mod.rs`.

Avoid vague files like `utils.rs`, `helpers.rs`, and `misc.rs`.

## Dialogs

All dialogs must use the shared dialog builder.

Do not duplicate dialog setup code.

The builder must handle titles, body text, buttons, suggested actions, destructive actions, cancellation, parent windows, defaults, keyboard behavior, and accessibility labels.

## GTK/libadwaita UI

- Use standard GTK/libadwaita components unless there is a strong reason not to.
- Follow GNOME Human Interface Guidelines.
- Keep Flathub packaging and sandboxing in mind.
- Avoid unnecessary custom widgets or styling.
- Keep UI accessible and keyboard-friendly.

## Buttons

- Only actions that really add or modify data can and must be suggested.
- There may be only one visible suggested button per screen, page, dialog, or popup.
- Dangerous actions must be destructive/danger.
- Cancel actions must be neutral.
- Navigation, view, filter, search, refresh, and menu actions must not be suggested.
- Header, toolbar, and footer buttons must be flat icon buttons.
- Only the one suggested button may also have text in a header.
- More than two adjacent buttons must be grouped.
- More than three grouped buttons must move into a real dotted menu.
- Keep only the most important button or buttons outside the menu.
- Use a real menu, not a popover, for overflow actions.

## Main thread

Only final GTK widget creation, final UI updates, and UI event handling belong on the GTK main thread.

Move IO, API calls, parsing, database work, long-running calculations, blocking waits, polling, and heavy UI preparation to workers, async tasks, channels, or threads.

If UI work is heavy or renders a lot of data, prepare it in the background and show a loading placeholder until it is ready.

Never update GTK widgets directly from a background thread.

## Data flow

Prefer clear data pipelines and message passing:

```text
UI event -> command -> worker -> result -> UI update
```

Keep mutation local and obvious.

## Dependencies

Do not add dependencies unless needed.

Prefer the Rust standard library, GTK, libadwaita, and existing project code first.

New dependencies must be maintained, license-compatible, and reasonable for Flathub.
