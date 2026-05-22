# Local AI Model Assets

These files are placeholders. Bank Files treats local AI as unavailable while
`model.json` contains `"placeholder": true`.

## Files To Replace

- `classifier.onnx`: a small ONNX model that `tract` can load for local
  classification, category hints, and pattern grouping hints.
- `generator.safetensors`: tiny Candle-compatible generator weights for JSON
  configuration drafts.
- `tokenizer.json`: tokenizer used by the generator model.
- `model.json`: manifest for the local model bundle. Set `placeholder` to
  `false` after real assets are installed, and add any model-specific metadata
  needed by the runtime adapter.
- `LICENSE`: license text for the real model assets. Only bundle assets that are
  allowed to be redistributed with Bank Files.

## Build And Install Modes

- `cargo build --features local-ai` copies `data/ai` beside the executable as
  `models/ai` for local development.
- Linux Meson installs with `local-ai` and without `setup` install the model
  bundle to `$datadir/bank-files/models/ai` and compile the binary with that
  data directory as an additional lookup path.
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

## TODO Before Real Inference Works

1. Pick tiny redistributable models and confirm their licenses allow bundling.
2. Define the final `model.json` schema: architecture, backend, input names,
   output names, tokenizer settings, and prompt/template version.
3. Replace the placeholder files and set `placeholder` to `false`.
4. Implement the model-specific `tract` classifier adapter in
   `src/local_ai/mod.rs`.
5. Implement the model-specific Candle generator adapter in
   `src/local_ai/mod.rs` so it returns strict JSON drafts.
6. Keep validating every generated budget, rule, alias, ignored pattern, and
   regex with the existing data-layer validators before writing anything.
7. Add golden tests with small fixture assets or mocked model output for valid
   drafts, invalid drafts, fallback behavior, and Smart Insights disabled.
8. Rebuild and test these paths:
   - `cargo test --locked --features local-ai`
   - `cargo test --locked --features embedded-ai-model`
   - `cargo test --locked --features setup`

## Current Implementation Status

The app already has the provider boundary, asset resolution, sidecar copying,
embedded asset support, sanitized inputs, strict draft validation, and safe
fallback to deterministic Smart Insights. The current runtime intentionally does
not run real inference yet because the placeholder files do not define a concrete
model architecture or output schema.
