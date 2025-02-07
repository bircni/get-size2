#![expect(dead_code, reason = "this is an example")]
use get_size2::GetSize;

type Buffer1024 = Vec<u8>;

#[derive(GetSize)]
struct TestStruct {
    id: u64,
    #[get_size(size = 1024)]
    buffer: Buffer1024, // Always allocates exactly 1KB at the heap.
}

fn main() {
    let test = TestStruct {
        id: 1,
        buffer: Buffer1024::new(),
    };

    assert_eq!(test.get_heap_size(), 1024, "TestStruct");
}
