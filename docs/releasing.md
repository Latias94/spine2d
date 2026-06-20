# Releasing

This repo is a workspace. Publish order matters because `spine2d-wgpu` and `spine2d-bevy`
depend on `spine2d`.

## Checklist

- Bump versions (`spine2d`, `spine2d-wgpu`, `spine2d-bevy`, workspace consumers).
- Update `CHANGELOG.md`.
- Run tests:
  - `cargo test -p spine2d --features json,binary,upstream-smoke`
  - `cargo check -p spine2d-bevy`
- Dry-run publish:
  - `cargo publish -p spine2d --dry-run`
  - `cargo publish -p spine2d-wgpu --dry-run` (requires `spine2d` already published)

`spine2d-bevy` is validated locally with `cargo check` and then published after `spine2d`
is live, because its packaged dependency on `spine2d` resolves through crates.io.

## Publish order

1. `spine2d`
2. `spine2d-wgpu`
3. `spine2d-bevy`
