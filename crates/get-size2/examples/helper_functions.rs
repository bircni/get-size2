use get_size2::GetSize;

type ExternalVecAlike<T> = Vec<T>;

#[derive(GetSize)]
struct TestStruct {
    id: u64,
    #[get_size(size_fn = vec_alike_helper)]
    buffer: ExternalVecAlike<u8>,
}

// NOTE: We assume that slice.len()==slice.capacity()
fn vec_alike_helper<V, T>(slice: &V) -> usize
where
    V: AsRef<[T]>,
{
    std::mem::size_of_val(slice.as_ref())
}

fn main() {
    let buffer = vec![0u8; 512];
    let buffer: ExternalVecAlike<u8> = buffer;

    let test = TestStruct { id: 1, buffer };

    assert_eq!(test.get_heap_size(), 512, "TestStruct");
}
