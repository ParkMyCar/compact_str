use crate::CompactString;
use bevy_reflect::{
    impl_reflect_opaque, std_traits::ReflectDefault, ReflectDeserialize, ReflectSerialize,
};

impl_reflect_opaque!((in crate::CompactString)CompactString(
    Clone,
    Debug,
    Hash,
    PartialEq,
    Default,
    Serialize,
    Deserialize,
));

#[cfg(test)]
mod tests {
    use bevy_reflect::{FromReflect, PartialReflect};

    use crate::CompactString;

    #[test]
    fn should_partial_eq_compactstring() {
        let a: &dyn PartialReflect = &CompactString::new("A");
        let a2: &dyn PartialReflect = &CompactString::new("A");
        let b: &dyn PartialReflect = &CompactString::new("B");
        assert_eq!(Some(true), a.reflect_partial_eq(a2));
        assert_eq!(Some(false), a.reflect_partial_eq(b));
    }

    #[test]
    fn compactstring_should_from_reflect() {
        let string = CompactString::new("hello_world.rs");
        let output = <CompactString as FromReflect>::from_reflect(&string);
        assert_eq!(Some(string), output);
    }

    #[test]
    fn compactstring_heap_should_from_reflect() {
        let string = CompactString::new("abc".repeat(100));
        let output = <CompactString as FromReflect>::from_reflect(&string);
        assert_eq!(Some(string), output);
    }
}
