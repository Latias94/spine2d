# Spine 4.3 Latest Tag Upstream Audit Map

This document anchors module-by-module parity work to the latest official Spine 4.3 runtime tag.

## Baseline

- Upstream repo: `https://github.com/EsotericSoftware/spine-runtimes`
- Target version: `4.3-latest-tag`
- Pinned ref: `spine-ts-4.3.8`
- Pinned commit: `8e12b1250ab88c0f890849ea45aab80338cead63`
- Primary reference path: `spine-cpp/`

## Why This Tag

Official 4.3 refs currently include:

- `spine-ts-4.3.8`
- `spine-libgdx-4.3.2`
- `spine-flutter-4.3.3`
- `spine-flutter-4.3.4`

There is no plain `4.3.2` tag in the official repository. Local verification shows no behaviour-relevant `spine-cpp` drift between the pinned tag and the current `4.3` branch.

The project now pins `spine-ts-4.3.8` because it is the newest official 4.3 tag. The comparison gate checks the moving `4.3` branch for drift in `spine-cpp`.

## Audit Workflow

For each runtime module:

1. Compare Rust behavior against `spine-cpp` first.
2. Add or update an oracle scenario before changing behavior when the difference is observable.
3. Record any intentional deviation in a follow-up decision document.

## Priority Areas

- AnimationState mixing and timeline property gating.
- JSON and binary data format fields introduced or stabilized in 4.3.
- Physics, slider, IK, transform, and path constraint update ordering.
- Slot attachment, draw order, event, deform, sequence, and clipping semantics.
