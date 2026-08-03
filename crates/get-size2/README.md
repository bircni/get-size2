# get-size2

[![Crates.io](https://img.shields.io/crates/v/get-size2)](https://crates.io/crates/get-size2)
[![docs.rs](https://img.shields.io/docsrs/get-size2)](https://docs.rs/get-size2)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/bircni/get-size2/blob/main/crates/get-size2/LICENSE)

Determine the size in bytes an object occupies inside RAM.

`size_of` already reports how many bytes a value occupies on the stack. Applications like caches or memory budgets also need to know how many bytes a value owns on the *heap*, which is what the `GetSize` trait adds.

> This crate is a fork of [get-size](https://github.com/DKerp/get-size), which is no longer maintained.

## Usage

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

Only bytes *owned* by a value are counted, so anything merely borrowed through a reference counts as zero. Shared ownership (`Rc`, `Arc`) is counted once, deduplicated by a tracker.

## Features

`std` is enabled by default and implies `alloc`; disable both for `no_std` targets:

```toml
get-size2 = { version = "^0.10", default-features = false, features = ["alloc", "derive"] }
```

| Feature | Description |
| ------- | ----------- |
| `std` _(default)_ | Implementations for `std` types like `HashMap`, `PathBuf` or `Mutex` |
| `alloc` | Implementations for `alloc` types like `Vec`, `String`, `Box`, `Rc` and `Arc` |
| `derive` | The `#[derive(GetSize)]` macro |
| `all-features-no-std` | Every feature which works without `std` |

Implementations for third party types are available behind the `bytes`, `chrono`, `chrono-tz`, `compact-str`, `dashmap`, `half`, `hashbrown`, `indexmap`, `ordermap`, `orx-concurrent-vec`, `parking_lot`, `portable-atomic`, `roaring`, `smallvec`, `thin-vec` and `url` features. `dashmap` and `parking_lot` require `std`.

## Documentation

The full guide is on [docs.rs](https://docs.rs/get-size2): the ownership model and its pitfalls, tracking shared ownership, `no_std` support, deriving and implementing the trait, and what the reported sizes do and do not include. The derive attributes are documented in [`get-size-derive2`](https://docs.rs/get-size-derive2).

## License

This library is licensed under the [MIT license](http://opensource.org/licenses/MIT).

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this library by you shall be licensed as MIT, without any additional terms or conditions.
