use super::Repr;

const FALSE: Repr = Repr::new_const("false");
const TRUE: Repr = Repr::new_const("true");

/// Defines how to _efficiently_ create a [`Repr`] from `self`
pub trait IntoRepr {
    fn into_repr(self) -> Repr;
}

impl IntoRepr for f32 {
    fn into_repr(self) -> Repr {
        let mut buf = ryu::Buffer::new();
        let s = buf.format(self);
        Repr::new(s)
    }
}

impl IntoRepr for f64 {
    fn into_repr(self) -> Repr {
        let mut buf = ryu::Buffer::new();
        let s = buf.format(self);
        Repr::new(s)
    }
}

impl IntoRepr for bool {
    fn into_repr(self) -> Repr {
        if self {
            TRUE
        } else {
            FALSE
        }
    }
}

impl IntoRepr for char {
    fn into_repr(self) -> Repr {
        let mut buf = [0_u8; 4];
        Repr::new_const(self.encode_utf8(&mut buf))
    }
}

impl IntoRepr for String {
    fn into_repr(self) -> Repr {
        Repr::from_string(self)
    }
}

impl IntoRepr for Box<str> {
    fn into_repr(self) -> Repr {
        Repr::from_box_str(self)
    }
}
