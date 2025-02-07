use get_size2::GetSize;

#[derive(GetSize)]
struct TestStructGenerics<A, B> {
    value1: A,
    value2: B,
}

#[derive(GetSize)]
enum TestEnumGenerics<A, B> {
    Variant1(A),
    Variant2(B),
}

fn main() {
    let test: TestStructGenerics<String, u64> = TestStructGenerics {
        value1: "Hello".into(),
        value2: 123,
    };

    assert_eq!(test.get_heap_size(), 5, "TestStructGenerics");

    let test = String::from("Hello");
    let test: TestEnumGenerics<String, u64> = TestEnumGenerics::Variant1(test);

    assert_eq!(test.get_heap_size(), 5, "TestEnumGenerics::Variant1");

    let test: TestEnumGenerics<String, u64> = TestEnumGenerics::Variant2(100);

    assert_eq!(test.get_heap_size(), 0, "TestEnumGenerics::Variant2");
}
