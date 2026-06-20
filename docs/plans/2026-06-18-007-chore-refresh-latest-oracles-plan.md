---
title: "chore: Refresh latest tag oracle baselines"
type: "chore"
date: "2026-06-18"
execution: "code"
---

# chore: Refresh latest tag oracle baselines

## Summary

Refresh the upstream cache, imported example assets, pose oracle goldens, and render oracle goldens so all parity signals are recorded against `spine-flutter-4.3.4` at `80dc680a4345ac09cdc5d4c1a77ec572a3f295d1`.

---

## Problem Frame

Runtime fixes have already moved several parser and animation surfaces toward latest Spine 4.3 behavior, but stale oracle outputs can still make upstream-smoke failures look like runtime drift. The local upstream cache was previously on `4.3@7fffd822fa17d924276d8727caa87fb98ccf015e` while the manifest and imported assets named `spine-flutter-4.3.4`, so oracle regeneration must first prove every input uses the same latest tag.

---

## Requirements

### Baseline inputs

- R1. `.cache/spine-runtimes`, `assets/spine-runtimes/SOURCE.txt`, and `spine-upstream.toml` must agree on `spine-flutter-4.3.4` and commit `80dc680a4345ac09cdc5d4c1a77ec572a3f295d1`.
- R2. Baseline validation must check the remote tag ref, not just local text.

### Oracle outputs

- R3. Pose oracle goldens must be rebuilt from the latest tag C++ oracle and latest tag assets, with `spine2d/tests/golden/SOURCE.txt` recording the tag and target commit.
- R4. Render oracle goldens must be rebuilt or marked failed from the latest tag C++ render oracle and latest tag assets, with per-format `SOURCE.txt` files recording the tag and target commit.

### Parity signal

- R5. Core `json,binary` tests must remain green after the baseline refresh.
- R6. Upstream-smoke failures after oracle refresh must be grouped into stale-golden script gaps, oracle generation failures, or real Rust runtime mismatches.

---

## Key Technical Decisions

- **KTD1. Treat `spine-flutter-4.3.4` as the reproducible latest-tag baseline:** GitHub's `spine-runtimes` 4.3 tags are runtime-specific, so this plan follows the user-requested latest tag currently recorded in `spine-upstream.toml` rather than the older moving `4.3` branch head.
- **KTD2. Refresh inputs before accepting new goldens:** Golden output is only trustworthy after `.cache/spine-runtimes` and imported assets are both verified at the target commit.
- **KTD3. Separate stale output from runtime mismatch:** Re-recorded goldens are a diagnostic step, not proof that all behavior is correct; upstream-smoke remains the acceptance gate for Rust parity.

---

## Implementation Units

### U1. Verify and refresh latest upstream inputs

- **Goal:** Make local upstream source and imported example assets deterministic for the selected latest tag.
- **Requirements:** R1, R2.
- **Dependencies:** None.
- **Files:** `spine-upstream.toml`, `assets/spine-runtimes/SOURCE.txt`, `scripts/check_spine_baseline.py`, `scripts/fetch_spine_runtimes_examples.py`.
- **Approach:** Use the existing baseline manifest and fetch script to update `.cache/spine-runtimes` and `assets/spine-runtimes`, then verify the remote tag ref and local HEAD match `80dc680a4345ac09cdc5d4c1a77ec572a3f295d1`.
- **Patterns to follow:** Existing `upstream_baseline.py` manifest loader and `check_spine_baseline.py` validation flow.
- **Test scenarios:** Remote ref lookup for `refs/tags/spine-flutter-4.3.4` returns the manifest commit; `.cache/spine-runtimes` HEAD points at the same commit; imported `SOURCE.txt` records the same commit.
- **Verification:** Baseline check passes with remote verification, and no source checkout mismatch remains.

### U2. Rebuild pose oracle goldens

- **Goal:** Replace stale pose oracle outputs with latest tag C++ oracle output.
- **Requirements:** R3, R6.
- **Dependencies:** U1.
- **Files:** `scripts/record_oracle_goldens.py`, `spine2d/tests/golden/SOURCE.txt`, `spine2d/tests/golden/oracle_scenarios/`, `spine2d/tests/golden/oracle_scenarios_skel/`.
- **Approach:** First re-record the known stale `dragon_flying_sequence_t0_25` scenario, then run the full pose oracle rebuild with keep-going so script failures and real mismatches can be separated.
- **Patterns to follow:** Existing `oracle_scenario_parity_tests.rs` scenario discovery and the C++ lite oracle runner.
- **Test scenarios:** The dragon sequence JSON and `.skel` oracle tests pass against the refreshed output; full pose rebuild reports either OK or a finite list of recording failures with updated source metadata.
- **Verification:** Targeted dragon oracle tests pass, and the full pose oracle source file records latest tag metadata.

### U3. Rebuild render oracle goldens

- **Goal:** Replace stale render oracle outputs with latest tag C++ render oracle output or mark generation failures accurately.
- **Requirements:** R4, R6.
- **Dependencies:** U1.
- **Files:** `scripts/record_oracle_render_goldens.py`, `spine2d/tests/golden/render_oracle_scenarios/`, `spine2d/tests/golden/render_oracle_scenarios_skel/`.
- **Approach:** Run render oracle recording for JSON and `.skel` formats with keep-going. Treat missing export files or oracle crashes as generation failures to fix before accepting render mismatches.
- **Patterns to follow:** Existing render oracle case manifests in `record_oracle_render_goldens.py`.
- **Test scenarios:** JSON render source metadata records `spine-flutter-4.3.4`; `.skel` render source metadata records the same commit or an explicit failure count; stale `Branch: 4.3` metadata disappears from current render outputs.
- **Verification:** Render oracle parity tests either pass or fail only with Rust runtime diffs against latest-tag goldens.

### U4. Run parity gates and classify remaining failures

- **Goal:** Establish the next runtime mismatch backlog from current, trustworthy oracle data.
- **Requirements:** R5, R6.
- **Dependencies:** U2, U3.
- **Files:** `spine2d/src/runtime/oracle_scenario_parity_tests.rs`, `spine2d/src/runtime/render_oracle_parity_tests.rs`, `docs/parity.md`, `docs/upstream-tests.md`.
- **Approach:** Run formatting, core `json,binary` nextest, and upstream-smoke nextest. Group remaining failures by module and upstream source area before creating the next fix plan.
- **Patterns to follow:** Current `docs/upstream-tests.md` result recording and existing failure grouping in parity docs.
- **Test scenarios:** Core tests pass; upstream-smoke failure count is lower or better classified than the pre-refresh 86-failure baseline; any persistent failure references the refreshed golden commit.
- **Verification:** A concise failure inventory identifies the next concrete runtime behavior plan.

---

## Scope Boundaries

- This plan does not change runtime behavior unless oracle regeneration exposes script defects that block trustworthy parity signals.
- This plan does not preserve compatibility with stale branch or beta goldens.
- This plan does not commit regenerated goldens automatically; commit grouping requires user confirmation.

---

## Risks & Dependencies

- Render oracle rebuilding can fail because latest tag example exports may not have a `.skel` counterpart for every JSON case.
- Full oracle regeneration may produce large diffs; those diffs are expected only when the source metadata proves they came from the latest tag.
- Upstream-smoke can still fail after a clean rebuild; those failures become the next runtime parity work rather than blockers for this baseline refresh.

---

## Sources / Research

- GitHub remote tags show `refs/tags/spine-flutter-4.3.4` at `80dc680a4345ac09cdc5d4c1a77ec572a3f295d1`.
- GitHub repository tags also show adjacent runtime-specific tags such as `spine-libgdx-4.3.2`, so latest-tag selection must remain explicit.
- `scripts/check_spine_baseline.py` verifies the manifest tag and commit against the remote.
- `scripts/record_oracle_goldens.py` and `scripts/record_oracle_render_goldens.py` read the imported asset commit through `upstream_baseline.commit_for_examples`.
