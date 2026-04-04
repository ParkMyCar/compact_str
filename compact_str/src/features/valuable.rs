use valuable::{Valuable, Value, Visit};

use crate::CompactString;

impl Valuable for CompactString {
    fn as_value(&self) -> Value<'_> {
        Value::String(self.as_str())
    }

    fn visit(&self, visit: &mut dyn Visit) {
        self.as_str().visit(visit);
    }
}
