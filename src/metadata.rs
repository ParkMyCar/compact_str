use bitflags::bitflags;

bitflags! {
    pub struct Discriminant: u8 {
        const HEAP = 0b00000000;
        const INLINE = 0b10000000;
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Metadata(u8);

impl Metadata {
    pub const fn new_heap() -> Self {
        Metadata(Discriminant::HEAP.bits())
    }

    pub fn new_inline(text: &str) -> Self {
        debug_assert!(text.len() <= !Discriminant::all().bits() as usize);

        Metadata::new(Discriminant::INLINE, text.len() as u8)
    }

    pub fn new(discriminant: Discriminant, data: u8) -> Self {
        debug_assert_eq!(
            data & Discriminant::all().bits(),
            0,
            "data has the leading bit set, which will be clobbered by the discriminant"
        );

        let mut metadata = data;

        // clear all the bits used by the Discriminant
        metadata &= !Discriminant::all().bits();
        // set the disciminant
        metadata |= discriminant.bits();

        Metadata(metadata)
    }

    pub fn discriminant(&self) -> Discriminant {
        Discriminant::from_bits_truncate(self.0)
    }

    pub fn data(&self) -> u8 {
        // return the underlying u8, sans any bits from the discriminant
        self.0 & !Discriminant::all().bits()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Discriminant,
        Metadata,
    };

    #[test]
    fn test_all_valid_values() {
        let discriminants = [Discriminant::HEAP, Discriminant::INLINE];

        for d in discriminants {
            for i in 0..=127 {
                let m = Metadata::new(d, i);
                assert_eq!(m.discriminant(), d);
                assert_eq!(m.data(), i);
            }
        }
    }

    #[test]
    fn test_all_invalid_values_panic() {
        let discriminants = [Discriminant::HEAP, Discriminant::INLINE];

        for d in discriminants {
            for i in 128..=u8::MAX {
                let res = std::panic::catch_unwind(|| {
                    Metadata::new(d, i);
                });
                assert!(res.is_err())
            }
        }
    }
}
