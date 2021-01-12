# async-shell

* [Cargo package](https://crates.io/crates/async-shell)

## Examples

* [smol](demos/smol/src/sample.rs)

## Dev

```
cargo test --all --all-features -- --nocapture && \
cargo +nightly clippy --all --all-features -- -D clippy::all && \
cargo fmt --all -- --check
```
