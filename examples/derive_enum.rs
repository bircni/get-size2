use get_size2::GetSize;

#[derive(GetSize)]
pub enum TestEnum {
    Variant1(u8, u16, u32),
    Variant2(String),
    Variant3,
    Variant4 { x: String, y: String },
}

#[derive(GetSize)]
pub enum TestEnumNumber {
    Zero = 0,
    One = 1,
    Two = 2,
}

fn main() {
    let test = TestEnum::Variant1(1, 2, 3);
    assert_eq!(test.get_heap_size(), 0);

    let test = TestEnum::Variant2("Hello".into());
    assert_eq!(test.get_heap_size(), 5);

    let test = TestEnum::Variant3;
    assert_eq!(test.get_heap_size(), 0);

    let test = TestEnum::Variant4 {
        x: "Hello".into(),
        y: "world".into(),
    };
    assert_eq!(test.get_heap_size(), 5 + 5);

    let test = TestEnumNumber::One;
    assert_eq!(test.get_heap_size(), 0);
}
