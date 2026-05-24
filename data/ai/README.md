# Local AI Model Assets

Bank Files ships a small, redistributable local AI bundle. `cargo run` builds
with the local AI feature by default, copies these files beside the executable,
and uses the bundled local adapter when Smart Insights are enabled.

## Files

- `classifier.onnx`: ONNX classifier asset reserved for model-backed category
  and pattern adapters.
- `generator.safetensors`: Candle-compatible generator weights reserved for
  model-backed JSON configuration drafts.
- `tokenizer.json`: tokenizer loaded by the runtime as an asset compatibility
  check.
- `model.json`: manifest for the local model bundle. `placeholder` must stay
  `false` for the checked-in bundle so Cargo builds treat local AI as available.
- `LICENSE`: license text for the bundled model assets. Only bundle assets that
  are allowed to be redistributed with Bank Files.

## Build And Install Modes

- `cargo build` and `cargo run` include `local-ai` through the default feature
  set and copy `data/ai` beside the executable as `models/ai`.
- Use `cargo build --no-default-features` for a lightweight build without Smart
  Insights or local AI.
- Linux Meson installs with default Cargo features install the model bundle to
  `$datadir/bank-files/models/ai` and compile the binary with that data
  directory as an additional lookup path.
- At runtime on Linux, Bank Files also checks `XDG_DATA_DIRS` plus the standard
  `/usr/local/share` and `/usr/share` locations for
  `bank-files/models/ai`.
- `cargo build --features embedded-ai-model` embeds the files in the executable.
- `cargo build --features setup` enables embedded model assets for a
  self-contained build.

Smart Insights is the user-facing switch. When Smart Insights is off, local AI
is off too.

## Privacy Shape

Local AI input must stay local and minimal. Use merchant/counterparty labels,
descriptions, tags, direction hints, category hints, and aggregate counts. Do not
send or persist raw CSV rows, account numbers, transaction IDs, or unrelated
personal fields as AI input.

## Current Runtime

The provider boundary, asset resolution, sidecar copying, embedded asset support,
sanitized inputs, strict draft validation, and safe fallback to deterministic
Smart Insights are implemented. The current adapter produces local configuration
drafts and transaction pattern hints from repeated sanitized transaction labels,
existing categories, and the same bundled category keyword tables used by Smart
Insights. Generated drafts are merged with deterministic configuration output and
validated by the data layer before anything is written.

The ONNX classifier and Candle generator assets remain bundled and license
verified so model-specific adapters can replace or extend the deterministic local
adapter without changing the install layout.
