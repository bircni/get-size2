# get-size2

[![Crates.io](https://img.shields.io/crates/v/get-size2)](https://crates.io/crates/get-size2)
[![docs.rs](https://img.shields.io/docsrs/get-size2)](https://docs.rs/get-size2)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/bircni/get-size2/blob/main/LICENSE)

Determine the size in bytes an object occupies inside RAM.

`size_of` already reports how many bytes a value occupies on the stack. Applications like caches or memory budgets also need to know how many bytes a value owns on the *heap*, which is what the `GetSize` trait adds.

> This repository is a fork of [get-size](https://github.com/DKerp/get-size), which is no longer maintained.

```toml
[dependencies]
get-size2 = { version = "^0.10", features = ["derive"] }
```

```rust
use get_size2::GetSize;

#[derive(GetSize)]
struct Data {
    name: String,
    id: u64,
}

let data = Data { name: "Hello".into(), id: 1 };

assert_eq!(data.get_heap_size(), 5);
assert_eq!(data.get_size(), Data::get_stack_size() + 5);
```

## Crates

| Crate | Description | Documentation |
| ----- | ----------- | ------------- |
| [`get-size2`](crates/get-size2) | The `GetSize` trait, its implementations and the trackers | [docs.rs](https://docs.rs/get-size2) |
| [`get-size-derive2`](crates/get-size-derive2) | The `#[derive(GetSize)]` macro, re-exported by `get-size2` | [docs.rs](https://docs.rs/get-size-derive2) |

The crate is `no_std` compatible and ships optional implementations for 16 third party crates, see the [get-size2 README](crates/get-size2#features).

## Contributing

Issues and pull requests are welcome. `cargo test --all --all-features`, `cargo clippy --all-targets --all-features -- -D warnings` and `cargo fmt --all` should pass; CI additionally builds a 32-bit and a bare metal `no_std` target.

## License

This project is licensed under the [MIT license](http://opensource.org/licenses/MIT).

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you shall be licensed as MIT, without any additional terms or conditions.
