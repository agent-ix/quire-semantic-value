---
id: FR-130
title: "Share one input-path admission record below QSL and quire-eval"
type: FR
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-126
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/FR-106
    type: references
---
# FR-130: Share one input-path admission record below QSL and quire-eval

## Description

`quire-semantic-value` SHALL hold the one shared input-path admission record
that QSL and `quire-eval` can both name without either depending on the other.
This record describes the
diagnostic; QSL [FR-106](ix://agent-ix/quire-spec-language/FR-106) continues to
own observation admission, its check order, and the decision to admit, refuse,
or return incomplete.

## Outputs

- A public, `no_std`-compatible `AdmissionRecord` containing a `CatalogCode`
  whose `code()` and `cause()` return the exact catalog pair; a
  `BTreeMap<&'static str, String>` of named input fields; an ordered
  `Vec<PathStep>`; and an `Option<quire_exact::ConstructionCause>` named
  `kernel`.
- A public `PathStep` with exactly `Field(String)`, `OptionValue`, and
  `Index(usize)` variants. Steps run outermost first from the checked field,
  parameter, or result to the nested value. An empty path names that checked
  value itself. A field called `"0"` remains `Field("0")`, distinct from
  `Index(0)`.

## Behavior

1. QSV SHALL preserve the exact catalog code and cause, each named field and
   value, each typed path step in order, and the kernel construction cause when
   building or returning an `AdmissionRecord`. It SHALL NOT render the typed
   path into a string or discard the kernel cause.
2. `quire-eval` SHALL carry this QSV record in its call-admission failure.
   `quire-eval` SHALL NOT declare a second copy of its code, cause, fields, path, or kernel
   fields. Its `CallAdmissionLocus` SHALL distinguish population nodes from
   node, member, and binder loci, naming the population at which the fault was
   observed. This locus identifies the evaluator's population context; it
   does not replace the record's input path.
3. The observation admission implementation SHALL populate the QSV record at
   the QSL FR-106 decision site where the original defect is classified. A
   call admission implementation SHALL populate the same record shape at its
   own `quire-eval` admission decision site. Passing either result through
   another layer SHALL retain the original record and failure kind, without
   reconstructing it from a rendered message or a catalog code alone.
4. An observation API reporting admission SHALL return a typed admitted-or-
   refused result. Its refused arm SHALL carry the original record and
   distinguish `Refused` from `Incomplete`; a successful arm SHALL carry the
   admitted authority. A query for a missing successful authority SHALL
   return the original refusal rather than only `None`.
5. QSV SHALL own only the shared record and path vocabulary. QSV SHALL NOT
   depend on QSL or implement QSL FR-106 admission behavior. `quire-eval` SHALL
   NOT depend on QSL to carry the record.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-130-AC-1 | A shared record built with `invalid_runtime_input`/`invalid-value`, fields for document, population, object and field, path `[Field("0"), OptionValue, Index(2)]`, and `Some(ConstructionCause::TypeMismatch)` reads back the same code, cause, fields, path and kernel variant. `Field("0")` remains distinct from `Index(0)`. | Test |
| FR-130-AC-2 | QSL and `quire-eval` both compile against QSV's one public record and path types; QSV and `quire-eval` have no dependency on QSL, and `quire-eval` defines no parallel input-path record fields. | Inspection |
| FR-130-AC-3 | For a population-level admission defect, `CallAdmissionLocus` names the population and the call failure retains the original QSV record. Existing node, member and binder loci still identify their respective nodes. | Test |
| FR-130-AC-4 | An observation refused at a nested value returns a typed refusal with its original code/cause, named input fields, typed path and kernel cause; an incomplete observation retains its distinct failure kind. A successful observation returns its admitted authority. The missing-authority query exposes the original refusal rather than only `None`. | Test |

## Dependencies

- [FR-126](./FR-126-own-the-evaluation-result-vocabulary.md) owns
  `CatalogCode`.
- QSL [FR-106](ix://agent-ix/quire-spec-language/FR-106) owns the observation
  admission semantics and QSpec FR-207 defines the typed nested path.
