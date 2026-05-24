# GitHub Copilot Instructions

This is a Rust GTK/libadwaita project.

The AI goal is to follow `CONTRIBUTING.md`.

Generated or modified code must follow `CONTRIBUTING.md` first, especially the rules for:

- DRY, SRP, KISS, and idiomatic Rust.
- Clear separation of concerns.
- Applied patterns such as builders, commands, services, workers, and pipelines.
- GTK/libadwaita and GNOME UI rules.
- Flathub-friendly design.
- Shared dialog builder usage.
- Suggested/destructive button rules.
- Header, toolbar, footer, grouped-button, and dotted-menu rules.
- Keeping Rust files under 300 lines.
- Moving heavy work off the GTK main thread.
- Showing loading placeholders while heavy UI work is prepared.

If a request conflicts with `CONTRIBUTING.md`, follow `CONTRIBUTING.md`.