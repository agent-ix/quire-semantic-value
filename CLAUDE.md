# quire-semantic-value

The shared no_std semantic-value leaf: runtime semantic values over the quire-exact kernel.

## Commands

```bash
make fmt            # format with rustfmt
make fmt-check      # verify formatting (CI gate)
make lint           # clippy with -D warnings
make test           # cargo test
make build          # release build
make clean          # cargo clean
make deny           # cargo deny check
make ci             # fmt-check + lint + test + no_std build + deny + docs
```

## Safety scaffolding

Backported from `agent-ix/ecaz`:

- `clippy.toml` pins MSRV to `1.98` and caps cognitive complexity / arg count
- `deny.toml` allow-lists licenses and denies unknown registries/git sources
- `rustfmt.toml` uses 100-char width and `StdExternalCrate` import grouping. CI fails on drift.
- `rust-toolchain.toml` pins Rust 1.98.1 with rustfmt, clippy and the `thumbv7em-none-eabi` target.

## Layout

```
src/lib.rs             # crate root; unit tests sit beside the code
spec/                  # the requirements this crate owns
```
