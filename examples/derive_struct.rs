use get_size2::GetSize;

#[derive(GetSize)]
pub struct OwnStruct {
    value1: String,
    value2: u64,
}

fn main() {
    let test = OwnStruct {
        value1: "Hello".into(),
        value2: 123,
    };

    assert_eq!(test.get_heap_size(), 5);
}
