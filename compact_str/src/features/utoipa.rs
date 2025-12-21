use crate::CompactString;
use std::vec::Vec;

#[cfg_attr(docsrs, doc(cfg(feature = "utoipa")))]
impl utoipa::__dev::ComposeSchema for CompactString {
    fn compose(
        _generics: Vec<utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>>,
    ) -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::ObjectBuilder::new()
            .schema_type(utoipa::openapi::schema::SchemaType::new(
                utoipa::openapi::schema::Type::String,
            ))
            .into()
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "utoipa")))]
impl utoipa::ToSchema for CompactString {
    #[inline]
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("CompactString")
    }

    #[inline]
    fn schemas(
        schemas: &mut Vec<(
            std::string::String,
            utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
        )>,
    ) {
        schemas.extend([]);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_schema_type() {
        assert_eq!(
            utoipa::schema!(std::string::String),
            utoipa::schema!(crate::CompactString)
        );
        assert_eq!(utoipa::schema!(str), utoipa::schema!(crate::CompactString));
        assert_eq!(utoipa::schema!(&str), utoipa::schema!(crate::CompactString));
    }
}
