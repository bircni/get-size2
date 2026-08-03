# get-size-derive2

[![Crates.io](https://img.shields.io/crates/v/get-size-derive2)](https://crates.io/crates/get-size-derive2)
[![docs.rs](https://img.shields.io/docsrs/get-size-derive2)](https://docs.rs/get-size-derive2)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/bircni/get-size2/blob/main/crates/get-size-derive2/LICENSE)

Derives the `GetSize` trait of [get-size2](https://crates.io/crates/get-size2) for structs and enums.

## Usage

This crate is re-exported by `get-size2`, so depend on that with the `derive` feature instead of adding it directly:

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

assert_eq!(Data { name: "Hello".into(), id: 1 }.get_heap_size(), 5);
```

The generated implementation sums up the heap size of every field, so every field type has to implement `GetSize` as well. Shared ownership is deduplicated, since the generated implementation threads a tracker through all fields. Unions are not supported.

## Attributes

For fields whose type does not implement `GetSize`, the `#[get_size(...)]` attribute offers three ways out, plus a struct or enum level escape hatch for generics:

| Attribute | Position | Effect |
| --------- | -------- | ------ |
| `#[get_size(ignore)]` | Field | Skips the field, contributing `0` |
| `#[get_size(size = 1024)]` | Field | Accounts the field with a fixed number of bytes |
| `#[get_size(size_fn = my_helper)]` | Field | Calls `my_helper(&field)` to determine the heap size |
| `#[get_size(ignore(A, B))]` | Struct or enum | Drops the `GetSize` bound on the listed generic types |

## Documentation

See [docs.rs](https://docs.rs/get-size-derive2) for the full attribute reference with examples, and [docs.rs/get-size2](https://docs.rs/get-size2) for the trait itself.

## License

This library is licensed under the [MIT license](http://opensource.org/licenses/MIT).

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this library by you shall be licensed as MIT, without any additional terms or conditions.
