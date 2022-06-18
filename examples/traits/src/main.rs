use compact_str::{
    CompactStringExt,
    ToCompactString,
};

fn main() {
    // CompactStringExt allows you to join collections to create a CompactString
    let names = ["Joe", "Bob", "Alice"];
    let compact = names.join_compact(", ");

    assert_eq!(compact, "Joe, Bob, Alice");
    println!("{}", compact);

    // CompactStringExt also allows you to directly concatenate collections
    let fruits = ["apple", "orange", "banana"];
    let compact = fruits.concat_compact();

    assert_eq!(compact, "appleorangebanana");
    println!("{}", compact);

    // ToCompactString allows you to convert individual types into a CompactString
    let number = 42;
    let compact = number.to_compact_string();

    assert_eq!(compact, "42");
    println!("{}", compact);

    let answer = true;
    let compact = answer.to_compact_string();

    assert_eq!(compact, "true");
    println!("{}", compact);
}
