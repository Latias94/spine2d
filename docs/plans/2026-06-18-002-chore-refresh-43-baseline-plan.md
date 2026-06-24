---
title: "chore: Refresh Spine 4.3 upstream baseline"
type: "chore"
date: "2026-06-18"
---

# chore: Refresh Spine 4.3 upstream baseline

## Summary

Refresh the project from the stale `4.3-beta` baseline language to a reproducible Spine 4.3 baseline pinned by upstream commit. Treat `spine-runtimes` branch `4.3` commit `7fffd822fa17d924276d8727caa87fb98ccf015e` as the runtime-core reference for this cycle, while recording that runtime-specific tags such as `spine-libgdx-4.3.2` are auxiliary release markers rather than the canonical core baseline.

---

## Problem Frame

The repository currently documents and scripts a `4.3-beta` default even though upstream has a live `4.3` branch and runtime-specific 4.3 tags. That makes oracle/golden refreshes ambiguous: failures may reflect real Rust runtime drift, old beta assets, or an accidental choice of a single runtime package tag.

---

## Requirements

- R1. Baseline documentation must name the selected upstream reference commit and explain why `spine-libgdx-4.3.2` is not the canonical whole-runtime baseline.
- R2. Asset import and oracle recording scripts must default to a reproducible 4.3 reference instead of `4.3-beta`.
- R3. Existing parity docs must keep beta history where useful but describe the current refresh path in 4.3 terms.
- R4. Verification must show the script defaults resolve the selected upstream commit and that the core Rust test suite still passes without upstream assets.
- R5. Any discovered post-refresh oracle/golden failures must be captured as follow-up parity work, not hidden by silently changing goldens.

---

## Key Technical Decisions

- **Pin by upstream branch commit, not by `spine-libgdx-4.3.2`:** the GitHub tags currently include runtime-specific 4.3 releases, while the `4.3` branch is newer than `spine-libgdx-4.3.2` and better represents shared `spine-cpp` / `spine-c` core behavior.
- **Keep the pin explicit in repo text:** scripts can accept tags or branches, but docs must record the exact commit used for parity work so future golden updates are reproducible.
- **Do not rewrite golden outputs in this unit:** first make the baseline selection and tooling deterministic, then run oracle/golden refresh as a visible follow-up so real behavior deltas stay reviewable.
- **Use nextest when available:** AGENTS.md asks Rust tests to prefer `cargo nextest`; fall back to `cargo test` only if nextest is unavailable.

---

## Implementation Units

### U1. Document the current 4.3 baseline

- **Goal:** Replace stale beta-as-current language with a clear current baseline record.
- **Requirements:** R1, R3
- **Dependencies:** none
- **Files:** `docs/decisions.md`, `docs/parity.md`, `docs/upstream-audit-4.3-latest.md`, `README.md`
- **Approach:** Keep historical beta docs intact where they describe prior work, but add current-baseline notes that name branch `4.3`, commit `7fffd822fa17d924276d8727caa87fb98ccf015e`, and explain the `spine-libgdx-4.3.2` tag caveat.
- **Patterns to follow:** `docs/decisions.md` target-version section; `docs/parity.md` baseline note.
- **Test scenarios:** Test expectation: none -- documentation-only change.
- **Verification:** A grep for `4.3-beta` should show only historical notes or explicit migration references, not current default instructions.

### U2. Update import and oracle script defaults

- **Goal:** Make local asset import and oracle recording default to the selected 4.3 baseline.
- **Requirements:** R2, R4
- **Dependencies:** U1
- **Files:** `scripts/fetch_spine_runtimes_examples.py`, `scripts/prepare_spine_runtimes_web_assets.py`, `scripts/record_oracle_goldens.py`, `scripts/record_oracle_render_goldens.py`
- **Approach:** Introduce a shared textual default of `4.3` or the selected commit where the script currently says `4.3-beta`; preserve user override via `--rev`.
- **Patterns to follow:** existing `--rev` argparse wiring in the fetch/prepare scripts; existing SOURCE text writing in record scripts.
- **Test scenarios:**
  - Running each script with `--help` shows a 4.3 default and no longer instructs users to fetch `4.3-beta`.
  - Fetch dry-run equivalent is unavailable, so command construction should be verified by a shallow temporary checkout resolving the intended `4.3` branch commit before any asset copy is committed.
- **Verification:** Script help output and a temporary `git ls-remote`/`git rev-parse` check demonstrate the default can resolve the selected baseline.

### U3. Add a baseline sanity check test or script guard

- **Goal:** Provide a cheap automated signal that docs/scripts agree on the selected baseline.
- **Requirements:** R2, R4
- **Dependencies:** U1, U2
- **Files:** `scripts/fetch_spine_runtimes_examples.py`, `scripts/prepare_spine_runtimes_web_assets.py`, `docs/parity.md`
- **Approach:** Prefer a small helper constant or script-level text check over a broad new framework. The useful seam is the baseline string used by import tooling and documented in parity docs.
- **Patterns to follow:** lightweight script validation style in existing Python scripts.
- **Test scenarios:**
  - The selected baseline string appears consistently in fetch and prepare help output.
  - The parity docs mention the same commit when describing the current baseline.
- **Verification:** A targeted shell check over the relevant files returns the same branch/commit strings.

### U4. Run tests and capture follow-up parity deltas

- **Goal:** Establish whether the baseline/tooling refresh exposed immediate local failures.
- **Requirements:** R4, R5
- **Dependencies:** U1, U2, U3
- **Files:** `docs/parity.md`, `docs/upstream-tests.md`, `CHANGELOG.md`
- **Approach:** Run formatting, the normal Rust test suite, and available upstream-smoke tests only if local assets exist for the selected commit. If upstream-smoke fails because goldens still target the old beta commit, record that as follow-up rather than editing goldens in this unit.
- **Patterns to follow:** README test commands; existing `docs/parity.md` re-record checklist.
- **Test scenarios:**
  - Core tests pass with default features and enabled JSON/Binary features.
  - Upstream smoke is either green for the selected commit or reports a documented baseline-refresh mismatch.
- **Verification:** Test output identifies the exact command and result, and docs state any follow-up parity deltas.
- **Result:** `python3 scripts/check_spine_baseline.py`, `python3 scripts/check_spine_baseline.py --verify-remote`, `cargo fmt --check`, `cargo nextest run -p spine2d --features json,binary`, and `cargo nextest run -p spine2d --features json,binary,upstream-smoke` passed on 2026-06-18. The upstream-smoke run still used local assets pinned to `d050ae66829ed5e46bb38690c83f792ffc2b3d8b`, so re-importing and re-recording for `7fffd822fa17d924276d8727caa87fb98ccf015e` remains follow-up work.

---

## Scope Boundaries

- This plan does not change runtime behavior in `spine2d/src/runtime/*`.
- This plan does not re-record committed oracle/golden JSON files.
- This plan does not claim `spine-libgdx-4.3.2` is wrong; it only demotes it to an auxiliary reference for core parity.

### Deferred to Follow-Up Work

- Re-record pose/render oracle goldens against the selected commit after the baseline tooling lands.
- Compare `spine-cpp`, `spine-ts/spine-core`, and `spine-libgdx` latest 4.3 behavior for any runtime-specific divergences found by refreshed oracle runs.
- Deepen runtime Module seams around animation mixing and skeleton constraint update once baseline failures are reproducible.

---

## Risks & Dependencies

- Upstream `4.3` is still moving, so the selected commit must stay explicit even if scripts default to branch `4.3`.
- Local upstream example assets are not fully committed; upstream-smoke verification depends on the developer's local `assets/spine-runtimes` or `.cache/spine-runtimes` checkout.
- Network access is required to resolve current upstream references, but implementation must keep working for offline users who already have a checkout.

---

## Sources / Research

- `git ls-remote --heads https://github.com/EsotericSoftware/spine-runtimes.git` showed branch `4.3` at `7fffd822fa17d924276d8727caa87fb98ccf015e`.
- `git ls-remote --tags https://github.com/EsotericSoftware/spine-runtimes.git` showed runtime-specific 4.3 tags including `spine-libgdx-4.3.2`, `spine-flutter-4.3.3`, and `spine-flutter-4.3.4`.
- `npm view @esotericsoftware/spine-core version dist-tags --json` showed npm `latest` as `4.3.7`, confirming that GitHub's runtime-specific tags do not fully describe all package latest states.
