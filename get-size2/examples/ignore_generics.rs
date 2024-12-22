#![allow(dead_code)]
use get_size2::GetSize;

#[derive(GetSize)]
#[get_size(ignore(B, C, D))]
struct TestStructHelpers<A, B, C, D> {
    value1: A,
    #[get_size(size = 100)]
    value2: B,
    #[get_size(size_fn = get_size_helper)]
    value3: C,
    #[get_size(ignore)]
    value4: D,
}

// Does not implement GetSize
struct NoGS {}

const fn get_size_helper<C>(_value: &C) -> usize {
    50
}

fn main() {
    let test: TestStructHelpers<String, NoGS, NoGS, u64> = TestStructHelpers {
        value1: "Hello".into(),
        value2: NoGS {},
        value3: NoGS {},
        value4: 123,
    };

    assert_eq!(test.get_heap_size(), 5 + 100 + 50);
}
