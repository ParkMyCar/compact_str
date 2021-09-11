use crate::repr::Repr;

pub struct SmartStr {
    repr: Repr,
}

impl SmartStr {
    #[inline]
    pub fn new<T: AsRef<str>>(text: T) -> Self {
        SmartStr {
            repr: Repr::new(text),
        }
    }

    #[inline(always)]
    pub fn as_str(&self) -> &str {
        self.repr.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::SmartStr;

    #[test]
    fn sanity_test() {
        let small_str = SmartStr::new("hello world");
        assert_eq!(small_str.as_str(), "hello world");

        let large_str = SmartStr::new("Lorem ipsum dolor sit amet");
        assert_eq!(large_str.as_str(), "Lorem ipsum dolor sit amet");
    }
}
