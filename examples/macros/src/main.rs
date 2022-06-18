use compact_str::format_compact;

fn main() {
    // compact_str provides the format_compact! macro, which can be in place of the format! macro

    let user = "Parker";
    let msg = format_compact!("Hello {}", user);
    println!("CompactString: {}", msg);

    // `msg` is only 12 characters, so it can be inlined!
    assert!(!msg.is_heap_allocated());

    // and it's equaivalent to if we used the format! macro
    let msg_std = format!("Hello {}", user);
    assert_eq!(msg, msg_std);
    println!("std::String: {}", msg);
}
