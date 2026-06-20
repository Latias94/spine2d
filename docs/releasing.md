# Releasing

This repo is a workspace. Publish order matters because `spine2d-wgpu` depends on `spine2d`.

## Checklist

- Bump versions (`spine2d`, `spine2d-wgpu`, workspace consumers).
- Update `CHANGELOG.md`.
- Run tests:
  - `cargo test -p spine2d --features json,binary,upstream-smoke`
- Dry-run publish:
  - `cargo publish -p spine2d --dry-run`
  - `cargo publish -p spine2d-wgpu --dry-run` (requires `spine2d` already published)

## Publish order

1. `spine2d`
2. `spine2d-wgpu`

