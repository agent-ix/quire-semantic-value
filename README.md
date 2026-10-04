# quire-semantic-value

The shared no_std semantic-value leaf: runtime semantic values over the quire-exact kernel.

## Specification

This repository's `spec/` covers two requirements of the crate: FR-106-AC-10, the
object closure's admission rules (TC-904), and FR-060-AC-5, that the crate mints no
`NodeKey`. The requirements of the other modules (`quantity`, `unit`, `declaration`,
`enumeration`, `location`, `checking`, `call`, `semantic_node`, `definition`,
`containment`, `loss` and `stop`) are owned by `agent-ix/quire-spec-language`
(for example its FR-089, FR-140 to FR-143, FR-149, FR-151 to FR-153 and NFR-011) and
by `agent-ix/quire-specification`. They are not yet written here; moving them into
this repository is remaining work.

## Build

```bash
make test
```

## License

AGPL-3.0-or-later
