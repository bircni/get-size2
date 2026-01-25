# get-size2

[![Crates.io](https://img.shields.io/crates/v/get-size2)](https://crates.io/crates/get-size2)
[![Crates.io](https://img.shields.io/crates/v/get-size-derive2)](https://crates.io/crates/get-size-derive2)
[![docs.rs](https://img.shields.io/docsrs/get-size2)](https://docs.rs/get-size2)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/bircni/get-size2/blob/main/LICENSE)

> This repo is a fork of get-size, as it is not maintained anymore. The original repo can be found [here](https://github.com/DKerp/get-size)

This repo contains two crates: `get-size2` and `get-size-derive2`.

## get-size2

Determine the size in bytes an object occupies inside RAM.

### Tracking shared ownership

To avoid double-counting shared ownership (e.g. `Rc`, `Arc`), use a tracker:

```rust
use get_size2::{GetSize, StandardTracker};

fn main() {
  let value = std::sync::Arc::new(String::from("hello"));
  let (heap_size, _tracker) = value.get_heap_size_with_tracker(StandardTracker::new());
  assert_eq!(heap_size, std::mem::size_of::<String>() + 5);
}
```

## get-size-derive2

The derive macro will provide a custom implementation of the [`get_heap_size`] method, which will simply call [`get_heap_size`] on all contained values and add the values up. This implies that by default all values contained in the struct or enum must implement the [`GetSize`] trait themselves.
