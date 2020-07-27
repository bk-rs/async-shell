# async-shell

* [Cargo package](https://crates.io/crates/async-shell)

## Examples

* [smol](demos/smol/src/sample.rs)

## Dev

```
cargo test --all-features --all -- --nocapture && \
cargo clippy --all -- -D clippy::all && \
cargo fmt --all -- --check
```

```
cargo build-all-features
cargo test-all-features --all
```
