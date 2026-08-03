Derives the [`GetSize`] trait for structs and enums.

Available through the `derive` feature of [`get-size2`](https://docs.rs/get-size2):

```toml
get-size2 = { version = "^0.10", features = ["derive"] }
```

The generated [`get_heap_size`] implementation calls [`get_heap_size`] on every field and sums up the results, which means every field type must implement [`GetSize`] as well, unless it is handled by one of the [attributes](#attributes) below. Shared ownership is deduplicated, since the generated implementation threads a tracker through all fields.

Unions are not supported and produce a compile error.

# Examples

Structs, tuple structs and enums are all supported:

```rust
use get_size2::GetSize;

#[derive(GetSize)]
struct Data {
    name: String,
    id: u64,
}

#[derive(GetSize)]
struct Wrapper(String);

#[derive(GetSize)]
enum Message {
    Empty,
    Text(String),
    Pair { left: String, right: String },
}

#[derive(GetSize)]
enum Level {
    Low = 0,
    High = 1,
}

let data = Data { name: "Hello".into(), id: 123 };
assert_eq!(data.get_heap_size(), 5);

assert_eq!(Wrapper("Hello".into()).get_heap_size(), 5);

assert_eq!(Message::Empty.get_heap_size(), 0);
assert_eq!(Message::Text("Hello".into()).get_heap_size(), 5);
assert_eq!(
    Message::Pair { left: "Hello".into(), right: "world".into() }.get_heap_size(),
    5 + 5
);

assert_eq!(Level::High.get_heap_size(), 0);
```

Generics work too. The generated implementation requires every generic type to implement [`GetSize`], which [can be changed](#ignoring-generic-types):

```rust
use get_size2::GetSize;

#[derive(GetSize)]
struct Pair<A, B> {
    first: A,
    second: B,
}

let pair: Pair<String, u64> = Pair { first: "Hello".into(), second: 123 };
assert_eq!(pair.get_heap_size(), 5);
```

# Attributes

If a field's type does not implement [`GetSize`], the `#[get_size(...)]` attribute offers three ways out, plus a struct or enum level escape hatch for generics:

| Attribute | Position | Effect |
| --------- | -------- | ------ |
| `#[get_size(ignore)]` | Field | Skips the field, contributing `0` |
| `#[get_size(size = 1024)]` | Field | Accounts the field with a fixed number of bytes |
| `#[get_size(size_fn = my_helper)]` | Field | Calls `my_helper(&field)` to determine the heap size |
| `#[get_size(ignore(A, B))]` | Struct or enum | Drops the [`GetSize`] bound on the listed generic types |

The three field attributes work on named and on tuple struct fields. On enum variants only `ignore`, and only on named fields, is honored.

## Ignoring fields

The idiomatic use is a shared allocation which is already accounted for elsewhere, when the [tracker](https://docs.rs/get-size2/latest/get_size2/trait.GetSizeTracker.html) based deduplication is not what you want:

```rust
use std::sync::Arc;
use get_size2::GetSize;

#[derive(GetSize)]
struct PrimaryStore {
    id: u64,
    shared_data: Arc<Vec<u8>>,
}

#[derive(GetSize)]
struct SecondaryStore {
    id: u64,
    #[get_size(ignore)]
    shared_data: Arc<Vec<u8>>,
}

let shared_data = Arc::new(Vec::with_capacity(1024));

let primary = PrimaryStore { id: 1, shared_data: Arc::clone(&shared_data) };
let secondary = SecondaryStore { id: 2, shared_data };

// The `Arc` also stores the `Vec`'s stack data on the heap.
assert_eq!(primary.get_heap_size(), Vec::<u8>::get_stack_size() + 1024);
assert_eq!(secondary.get_heap_size(), 0);
```

It also works as a band aid for a field whose type does not implement [`GetSize`], but the result will then be too low unless that type really owns no heap memory. Prefer `size` or `size_fn` in that case.

```rust
use get_size2::GetSize;

// Does not implement GetSize!
struct External {
    value: String,
}

#[derive(GetSize)]
struct Data {
    name: String,
    #[get_size(ignore)]
    external: External,
}

let data = Data {
    name: "Adam".into(),
    external: External { value: "Hello world!".into() },
};

// The 12 bytes owned by `external` are missing from the result.
assert_eq!(data.get_heap_size(), 4);
```

## Returning a fixed size

For external types which always allocate the same number of bytes:

```rust
use get_size2::GetSize;
# struct Buffer1024 {}
#
# impl Buffer1024 {
#   fn new() -> Self {
#      Self {}
#   }
# }

#[derive(GetSize)]
struct Data {
    id: u64,
    #[get_size(size = 1024)]
    buffer: Buffer1024, // Always allocates exactly 1KB on the heap.
}

let data = Data { id: 1, buffer: Buffer1024::new() };
assert_eq!(data.get_heap_size(), 1024);
```

## Using a helper function

If the heap size of an external type can be derived from its public API, `size_fn` names a function to call. Unlike in other crates the function name is given directly, not as a string. This is particularly useful for a helper which is generic over a trait, and covers several types at once.

Alternatively, wrap the type in a newtype and implement [`GetSize`] for that.

```rust
use get_size2::GetSize;
# type ExternalVecAlike<T> = Vec<T>;

#[derive(GetSize)]
struct Data {
    id: u64,
    #[get_size(size_fn = vec_alike_helper)]
    buffer: ExternalVecAlike<u8>,
}

// NOTE: We assume that slice.len() == slice.capacity()
fn vec_alike_helper<V, T>(slice: &V) -> usize
where
    V: AsRef<[T]>,
{
    std::mem::size_of::<T>() * slice.as_ref().len()
}

let data = Data { id: 1, buffer: vec![0u8; 512].into() };
assert_eq!(data.get_heap_size(), 512);
```

## Ignoring generic types

A generic used only by ignored or helper handled fields still gets a [`GetSize`] bound, which would make the implementation unusable. List those generics in a struct level `ignore` to drop their bounds:

```rust
use get_size2::GetSize;

#[derive(GetSize)]
#[get_size(ignore(B, C, D))]
struct Data<A, B, C, D> {
    value1: A,
    #[get_size(size = 100)]
    value2: B,
    #[get_size(size_fn = helper)]
    value3: C,
    #[get_size(ignore)]
    value4: D,
}

// Does not implement GetSize.
struct NoGetSize {}

fn helper<C>(_value: &C) -> usize {
    50
}

let data: Data<String, NoGetSize, NoGetSize, u64> = Data {
    value1: "Hello".into(),
    value2: NoGetSize {},
    value3: NoGetSize {},
    value4: 123,
};

assert_eq!(data.get_heap_size(), 5 + 100 + 50);
```

# Errors

The macro reports a compile error when it is used on a union, or when conflicting attributes such as `size` and `ignore` are combined on the same field. A field whose type does not implement [`GetSize`] and which is not covered by an attribute also fails to compile.

[`GetSize`]: https://docs.rs/get-size2/latest/get_size2/trait.GetSize.html
[`get_heap_size`]: https://docs.rs/get-size2/latest/get_size2/trait.GetSize.html#method.get_heap_size
