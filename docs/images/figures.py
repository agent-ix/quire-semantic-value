# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 Agent-IX
"""quire-semantic-value's documentation figures, rendered by the docs-figures skill.

Regenerate from the repository root with the agent-ix/dev-tools skill:

    python3 -m pip install fonttools
    python3 <dev-tools>/skills/docs-figures/scripts/render.py docs/images/figures.py

Add --check to confirm the committed SVGs are current. The quantity flow is read
from src/quantity.rs and src/stop.rs in this repository.
"""

from docs_figures import FONTS, SMALL, TITLE, Figure  # noqa: F401

STYLE = "ix-docs"

# One role per kind of thing, fixed across the Quire repositories.
ROLES = [
    ("code", "authored source", "box"),
    ("logic", "compiler stage", "box"),
    ("model", "checked package / IR", "box"),
    ("evidence", "outcome", "box"),
    ("ink", "input", "box"),
]


# ---- figures -----------------------------------------------------------------


def hero():
    """README hero: what a quantity carries and how an operation ends."""
    f = Figure(
        "hero",
        980,
        330,
        "A quantity is an exact magnitude plus a unit id, not a bare number. A unit table resolves "
        "the id to a dimension and a conversion path. An operation first type-checks the units; "
        "mismatched dimensions are ill-typed and consume nothing. Otherwise it is evaluated under "
        "a meter and ends as completed (a value plus any loss record), undefined, refused or "
        "incomplete.",
    )
    f.box(0, 10, 200, 56, "ink", "quantity a", "exact magnitude · unit id")
    f.box(0, 90, 200, 56, "ink", "quantity b", "exact magnitude · unit id")
    f.box(0, 200, 200, 56, "model", "unit table", "id → dimension · path")
    f.box(270, 90, 170, 56, "logic", "type check", "same dimension?")
    f.line([(200, 38), (235, 38), (235, 108), (267, 108)], "ink")
    f.line([(200, 118), (267, 118)], "ink")
    f.line([(200, 228), (235, 228), (235, 128), (267, 128)], "model")
    f.box(270, 220, 170, 56, "evidence", "ill-typed", "nothing charged")
    f.line([(355, 146), (355, 217)], "evidence")
    f.box(510, 90, 170, 56, "logic", "evaluate", "exact · under a meter")
    f.line([(440, 118), (507, 118)], "logic")
    f.line([(680, 118), (705, 118)], "logic", arrow=False)
    f.line([(705, 38), (705, 296)], "logic", arrow=False)
    outs = [
        ("logic", "completed", "value · loss records"),
        ("evidence", "undefined", "no value exists"),
        ("evidence", "refused", "input rejected"),
        ("evidence", "incomplete", "meter ran out"),
    ]
    for i, (role, name, what) in enumerate(outs):
        y = 10 + i * 86
        f.line([(705, y + 28), (737, y + 28)], role)
        f.box(740, y, 240, 56, role, name, what)
    f.legend(0, 326, [ROLES[4], ROLES[1], ROLES[2], ("evidence", "outcome", "box")])
    return f


REPOS = [
    ("ORCHESTRATION", [("quire-verification", True), ("quire-driver", True), ("quire-integration", True)]),
    ("BACKENDS", [("quire-contract-codegen", False), ("quire-contract-runtime", False),
                  ("quire-analyze", False), ("quire-protocol", True), ("quire-mltl", False)]),
    ("LANGUAGE", [("quire-specification", True), ("quire-spec-language", False), ("quire-contract-ir", False)]),
    ("FOUNDATIONS", [("quire-exact", False), ("quire-semantic-value", False), ("quire-walk", False),
                     ("quire-canonical", False), ("quire-verification-contracts", False),
                     ("quire-observation", False)]),
]


def ecosystem(here="quire-semantic-value"):
    """The Quire repositories in bands; each band builds on the ones below."""
    f = Figure(
        "ecosystem",
        980,
        364,
        "The Quire repositories in four bands. Foundations: quire-exact, quire-semantic-value, "
        "quire-walk, quire-canonical, quire-verification-contracts, quire-observation. Language: "
        "quire-specification, quire-spec-language, quire-contract-ir. Backends: "
        "quire-contract-codegen, quire-contract-runtime, quire-analyze, quire-protocol, "
        "quire-mltl. Orchestration: quire-verification, quire-driver, quire-integration.",
    )
    y = 0
    gap = 10
    for band, repos in REPOS:
        f.text(0, y + 12, band, "muted", "mono", SMALL)
        widths = [FONTS["title"].width(name, TITLE) + 24 for name, _ in repos]
        spare = (980 - sum(widths) - gap * (len(repos) - 1)) / len(repos)
        x = 0
        for (name, private), w in zip(repos, widths):
            w = round(w + spare)
            role = "logic" if name == here else "ink"
            f.box(x, y + 20, w, 50, role, name, "private" if private else "public", dashed=private)
            x += w + gap
        y += 88
    f.legend(0, 360, [("logic", "this repository", "box"), ("ink", "private", "dash")])
    return f


FIGURES = [hero, ecosystem]
