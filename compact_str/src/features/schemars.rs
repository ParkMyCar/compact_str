use alloc::borrow::Cow;
use alloc::string::String;

use schemars::Schema;
use schemars::SchemaGenerator;

use crate::CompactString;

#[cfg_attr(docsrs, doc(cfg(feature = "schemars")))]
impl schemars::JsonSchema for CompactString {
    fn inline_schema() -> bool {
        <String as schemars::JsonSchema>::inline_schema()
    }

    fn schema_name() -> Cow<'static, str> {
        <String as schemars::JsonSchema>::schema_name()
    }

    fn schema_id() -> Cow<'static, str> {
        <String as schemars::JsonSchema>::schema_id()
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        <String as schemars::JsonSchema>::json_schema(generator)
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::String;

    use schemars::JsonSchema;

    use crate::CompactString;

    #[test]
    fn test_schema_matches_string() {
        let compact_name = CompactString::schema_name();
        let string_name = String::schema_name();
        assert_eq!(compact_name, string_name);

        let compact_id = CompactString::schema_id();
        let string_id = String::schema_id();
        assert_eq!(compact_id, string_id);

        let mut gen = schemars::SchemaGenerator::default();
        let compact_schema = CompactString::json_schema(&mut gen);
        let string_schema = String::json_schema(&mut gen);
        assert_eq!(compact_schema, string_schema);
    }
}
