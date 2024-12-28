use get_size2::GetSize;

#[derive(GetSize)]
struct Test<'a> {
    value: &'a String,
}

fn main() {
    let value = String::from("hello");

    // This string occupies 5 bytes at the heap, but a pointer is treated as not occupying
    // anything at the heap.
    assert_eq!(value.get_heap_size(), 5);
    assert_eq!(GetSize::get_heap_size(&&value), 0); // Fully qualified syntax

    // WARNING: Duo to rust's automatic dereferencing, a simple pointer will be dereferenced
    // to the original value, causing the borrowed bytes to be accounted for too.
    assert_eq!(value.get_heap_size(), 5);
    // The above gets rewritten by to compiler into:
    // assert_eq!(value.get_heap_size(), 5);

    // Our derive macro uses fully qualified syntax, so auto-dereferencing does
    // not occur.
    let value = Test { value: &value };

    // The String is now only borrowed, leading to its heap bytes not being
    // accounted for.
    assert_eq!(value.get_heap_size(), 0);
}
