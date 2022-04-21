use super::CompactStr;

use castaway::cast;

pub(super) fn to_compact_str_specialised<T>(val: &T) -> Option<CompactStr> {
    if let Ok(boolean) = cast!(val, &bool) {
        Some(CompactStr::new(if *boolean { "true" } else { "false" }))
    } else if let Ok(string) = cast!(val, &String) {
        Some(CompactStr::new(&*string))
    } else if let Ok(character) = cast!(val, &char) {
        Some(CompactStr::new(character.encode_utf8(&mut [0; 4][..])))
    } else {
        None
    }
}
