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
        "A quantity is an exact magnitude plus a unit id. A unit table resolves "
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
    (1, 0, "quire-driver", "the quire command", True),
    (2, 0, "quire-integration", "tests the whole chain", True),
    (3, 0, "quire-verification", "plans verification", True),
    (0, 1, "quire-specification", "the standard", True),
    (1, 1, "quire-spec-language", "compiler · evaluator", False),
    (2, 1, "quire-contract-ir", "contract model", False),
    (3, 1, "quire-contract-codegen", "code · tests · proofs", False),
    (4, 1, "quire-contract-runtime", "runs generated checks", False),
    (0, 2, "quire-observation", "qualified observations", False),
    (1, 2, "quire-protocol", "protocol conformance", True),
    (2, 2, "quire-analyze", "SMT consistency", False),
    (0, 3, "quire-mltl", "MLTL over observations", False),
]
FEEDS = [
    ("quire-specification", "quire-spec-language"),
    ("quire-spec-language", "quire-contract-ir"),
    ("quire-contract-ir", "quire-contract-codegen"),
    ("quire-contract-codegen", "quire-contract-runtime"),
    ("quire-spec-language", "quire-protocol"),
    ("quire-contract-ir", "quire-analyze"),
    ("quire-observation", "quire-protocol"),
    ("quire-observation", "quire-mltl"),
]
SHARED = ["quire-exact", "quire-semantic-value", "quire-walk", "quire-canonical",
          "quire-verification-contracts"]


def ecosystem(here="quire-semantic-value"):
    """The Quire repositories: the main chain, its branches, and what runs and shares it."""
    f = Figure(
        "ecosystem",
        980,
        476,
        "The Quire repositories. The main chain runs from the standard, quire-specification, to "
        "the compiler, quire-spec-language, then quire-contract-ir, quire-contract-codegen and "
        "quire-contract-runtime. quire-spec-language also feeds quire-protocol, and "
        "quire-contract-ir feeds quire-analyze. quire-observation feeds quire-protocol and "
        "quire-mltl. Above the chain, quire-driver runs it, quire-integration tests it and "
        "quire-verification plans verification. Below it, shared libraries used throughout: "
        "quire-exact, quire-semantic-value, quire-walk, quire-canonical and "
        "quire-verification-contracts.",
    )
    w, h, step = 176, 50, 201
    rows = {0: 20, 1: 120, 2: 210, 3: 290}
    where = {}
    for col, row, name, sub, private in REPOS:
        x, y = col * step, rows[row]
        where[name] = (x, y)
        role = "logic" if name == here else "ink"
        f.box(x, y, w, h, role, name, sub, dashed=private)
    for a, b in FEEDS:
        (ax, ay), (bx, by) = where[a], where[b]
        if ay == by:
            f.line([(ax + w, ay + h / 2), (bx - 3, by + h / 2)], "model")
        else:
            f.line([(ax + w / 2, ay + h), (bx + w / 2, by - 3)], "model")
    f.text(0, 12, "ORCHESTRATION", "muted", "mono", SMALL)
    f.text(0, 112, "THE CHAIN", "muted", "mono", SMALL)
    f.text(0, 382, "SHARED LIBRARIES · USED THROUGHOUT", "muted", "mono", SMALL)
    gap = 10
    widths = [FONTS["title"].width(name, TITLE) + 24 for name in SHARED]
    spare = (980 - sum(widths) - gap * (len(SHARED) - 1)) / len(SHARED)
    x = 0
    for name, w0 in zip(SHARED, widths):
        w0 = round(w0 + spare)
        f.box(x, 390, w0, h, "logic" if name == here else "ink", name)
        x += w0 + gap
    f.legend(0, 472, [("logic", "this repository", "box"), ("ink", "private", "dash"),
                      ("model", "feeds", "fill")])
    return f


FIGURES = [hero, ecosystem]
