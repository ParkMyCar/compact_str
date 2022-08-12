use core::fmt::{
    self,
    Write,
};

#[inline(always)]
pub(crate) fn count(args: impl fmt::Display) -> usize {
    let mut sink = Sink(0);
    write!(&mut sink, "{}", args).unwrap();
    sink.0
}

/// A special kind of sink that records the size of bytes writen into it.
struct Sink(usize);

impl Write for Sink {
    #[inline(always)]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0 += s.len();
        Ok(())
    }

    #[inline(always)]
    fn write_char(&mut self, c: char) -> fmt::Result {
        self.0 += c.len_utf8();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use core::format_args;

    use super::count;

    macro_rules! count {
        ( $fmt:expr $(, $args:tt)* ) => {
            count(
                format_args!(
                    $fmt,
                    $(
                        $args,
                    )*
                )
            )
        };
    }

    #[test]
    fn test_count() {
        assert_eq!(5, count!("{}", "12345"));
        assert_eq!(6, count!("1{}", "12345"));
        assert_eq!(7, count!("1{}{}", "12345", 2));
        assert_eq!(8, count!("1{}{}{}", "12345", 2, '2'));
        assert_eq!(12, count!("1{}{}{}{}", "12345", 2, '2', 1000));
    }
}
