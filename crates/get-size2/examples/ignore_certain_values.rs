#![expect(dead_code, reason = "this is an example")]
use get_size2::GetSize;
use std::sync::Arc;

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

fn main() {
    let shared_data = Arc::new(Vec::with_capacity(1024));

    let primary_data = PrimaryStore {
        id: 1,
        shared_data: Arc::clone(&shared_data),
    };

    let secondary_data = SecondaryStore { id: 2, shared_data };

    // Note that Arc does also store the Vec's stack data on the heap.
    assert_eq!(
        primary_data.get_heap_size(),
        Vec::<u8>::get_stack_size() + 1024,
        "PrimaryStore"
    );
    assert_eq!(secondary_data.get_heap_size(), 0, "SecondaryStore");
}
