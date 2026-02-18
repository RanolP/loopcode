# AGENTS.md

## Scope
- Rust workspace.
- Root app: `app`.
- Internal crates: `crates/*`.

## UI architecture
- `app` depends on `xpui` only.
- `xpui` is backend-agnostic (`UiApp`, neutral node tree, adapters).
- `cpui` is terminal backend (crossterm + taffy).

## Runtime
- CLI uses `clap`.
- `--graphics` => `xpui::run_gpui(...)`.
- default => `xpui::run_cpui(...)`.

## Code style
- Keep files modular and small.
- Keep backend internals behind `xpui`.

## Error handling
Reference: https://fast.github.io/blog/stop-forwarding-errors-start-designing-them/

- Design errors for caller action, not passthrough.
- At public boundaries, map dependency errors to domain errors.
- Keep machine-facing errors stable (kind/code/retryability).
- Keep human-facing errors contextual (operation + identifiers).
- Preserve source/cause internally; expose actionable domain error externally.
