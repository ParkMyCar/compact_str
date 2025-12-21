use crate::CompactString;

#[cfg_attr(docsrs, doc(cfg(feature = "utoipa")))]
impl utoipa::PartialSchema for CompactString {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::ObjectBuilder::new()
            .schema_type(utoipa::openapi::schema::SchemaType::new(
                utoipa::openapi::schema::Type::String,
            ))
            .into()
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "utoipa")))]
impl utoipa::ToSchema for CompactString {}

#[cfg(test)]
mod tests {
    use std::string::String;
    use utoipa::PartialSchema;

    #[test]
    fn test_compact_string_schema_matches_string() {
        let compact_schema = <crate::CompactString as PartialSchema>::schema();
        let string_schema = <String as PartialSchema>::schema();

        assert!(compact_schema == string_schema);
    }
}
