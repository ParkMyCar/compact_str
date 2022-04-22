use core::fmt;
use std::io::{Result, Write};

/// A special kind of sink that records the size of bytes writen into it.
#[derive(Debug, Default)]
pub(crate) struct Sink(usize);
impl Sink {
    #[inline(always)]
    pub(crate) fn count(args: fmt::Arguments) -> usize {
        let mut sink = Sink::default();
        write!(&mut sink, "{}", args).unwrap();
        sink.0
    }
}
#[macro_export]
macro_rules! count {
    ( $fmt:expr $(, $args:tt)* ) => {
        $crate::utility::Sink::count(
            core::format_args!(
                $fmt,
                $(
                    $args,
                )*
            )
        )
    };
}

impl Write for Sink {
    #[inline(always)]
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.0 += buf.len();
        Ok(buf.len())
    }

    #[inline(always)]
    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_count() {
        assert_eq!(5, count!("{}", "12345"));
        assert_eq!(6, count!("1{}", "12345"));
        assert_eq!(7, count!("1{}{}", "12345", 2));
        assert_eq!(8, count!("1{}{}{}", "12345", 2, '2'));
        assert_eq!(12, count!("1{}{}{}{}", "12345", 2, '2', 1000));
    }
}
