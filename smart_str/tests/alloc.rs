use smart_str::SmartStr;
use tracing::TracingAllocator;

#[global_allocator]
pub static ALLOCATOR: TracingAllocator = TracingAllocator::new();

#[test]
fn test_allocations() {
    ALLOCATOR.enable();
    {
        let small_str = SmartStr::new("hello world");
        assert_eq!(small_str.as_str(), "hello world");

        let large_str = SmartStr::new("Lorem ipsum dolor sit amet");
        assert_eq!(large_str.as_str(), "Lorem ipsum dolor sit amet");
    }
    ALLOCATOR.disable();

    // there should be two allocations events, one allocation to create the `large_str` and another
    // to free it once it goes out of scope. We shouldn't need to alloc any mem for `small_str`
    let events = ALLOCATOR.events();
    assert_eq!(events.len(), 2);

    let total_mem = events.iter().fold(0, |mem, event| mem +  event.delta());
    assert_eq!(total_mem, 0);
}
