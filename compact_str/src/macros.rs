#[macro_export]
macro_rules! format_compact {
    ($fmt:expr) => {{ $crate::ToCompactString::to_compact_string(&$fmt) }};
    ($fmt:expr, $($args:tt)*) => {{
        $crate::ToCompactString::to_compact_string(&format_args!($fmt, $($args)*))
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
        assert_eq!(format_compact!(2), "2");
        assert_eq!(format_compact!("{}", 2), "2");
    }
}
