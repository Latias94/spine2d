#!/usr/bin/env python3
# -*- coding: utf-8 -*-

from __future__ import annotations

import argparse
import re
from pathlib import Path


PATCH_MARKER = "SPINE2D_ORACLE_PATCH_SLIDER_SORT_SLOT_TIMELINE_CAST"


def patch_spine_cpp_slider_cpp(text: str) -> str:
    if PATCH_MARKER in text:
        return text

    # Newer 4.3 runtimes already contain the slot timeline dispatch this script used to patch in.
    # Keep the oracle build path idempotent across both older beta snapshots and latest 4.3.
    if "SlotTimeline *asSlotTimeline(Timeline *timeline)" in text and (
        "SlotTimeline *slotTimeline = asSlotTimeline(t)" in text
    ):
        return text

    if "#include <spine/SlotTimeline.h>" not in text:
        raise RuntimeError("Unexpected Slider.cpp: missing include <spine/SlotTimeline.h>")

    if "#include <spine/SlotCurveTimeline.h>" not in text:
        text = text.replace(
            "#include <spine/SlotTimeline.h>",
            "\n".join(
                [
                    "#include <spine/SlotTimeline.h>",
                    "#include <spine/SlotCurveTimeline.h>",
                    "#include <spine/AttachmentTimeline.h>",
                    "#include <spine/SequenceTimeline.h>",
                    f"// {PATCH_MARKER}",
                ]
            ),
        )

    pattern = re.compile(
        r"""
        if\s*\(t->getRTTI\(\)\.instanceOf\(SlotTimeline::rtti\)\)\s*\{\s*
            SlotTimeline\s*\*\s*timeline\s*=\s*\(SlotTimeline\s*\*\)\s*t\s*;\s*
            skeleton\.constrained\(\*slots\[timeline->getSlotIndex\(\)\]\)\s*;\s*
        \}
        """,
        re.VERBOSE,
    )

    replacement = "\n".join(
        [
            "if (t->getRTTI().instanceOf(SlotTimeline::rtti)) {",
            "\t\t\t\tint slotIndex = -1;",
            "\t\t\t\tif (t->getRTTI().instanceOf(SlotCurveTimeline::rtti)) {",
            "\t\t\t\t\tslotIndex = static_cast<SlotCurveTimeline *>(t)->getSlotIndex();",
            "\t\t\t\t} else if (t->getRTTI().instanceOf(AttachmentTimeline::rtti)) {",
            "\t\t\t\t\tslotIndex = static_cast<AttachmentTimeline *>(t)->getSlotIndex();",
            "\t\t\t\t} else if (t->getRTTI().instanceOf(SequenceTimeline::rtti)) {",
            "\t\t\t\t\tslotIndex = static_cast<SequenceTimeline *>(t)->getSlotIndex();",
            "\t\t\t\t}",
            "\t\t\t\tif (slotIndex != -1) skeleton.constrained(*slots[slotIndex]);",
            "\t\t\t}",
        ]
    )

    text2, n = pattern.subn(replacement, text, count=1)
    if n != 1:
        raise RuntimeError("Failed to patch Slider.cpp: pattern not found (upstream changed?)")
    return text2


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--in", dest="inp", required=True, help="Input Slider.cpp path")
    ap.add_argument("--out", dest="out", required=True, help="Output patched Slider.cpp path")
    args = ap.parse_args()

    inp = Path(args.inp)
    out = Path(args.out)
    src = inp.read_text(encoding="utf-8")
    patched = patch_spine_cpp_slider_cpp(src)

    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(patched, encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
