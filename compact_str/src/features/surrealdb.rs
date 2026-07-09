use crate::CompactString;
use std::string::ToString;

#[cfg_attr(docsrs, doc(cfg(feature = "surrealdb")))]
impl surrealdb_types::SurrealValue for CompactString {
    fn kind_of() -> surrealdb_types::Kind {
        surrealdb_types::Kind::String
    }

    fn into_value(self) -> surrealdb_types::Value {
        surrealdb_types::Value::String(self.into_string())
    }

    fn from_value(value: surrealdb_types::Value) -> Result<Self, surrealdb_types::Error>
    where
        Self: Sized,
    {
        match value {
            surrealdb_types::Value::String(v) => Ok(CompactString::new(v)),
            _ => Err(surrealdb_types::Error::thrown(
                "surrealdb value not string".to_string(),
            )),
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "surrealdb")))]
impl From<CompactString> for surrealdb_types::Value {
    fn from(item: CompactString) -> Self {
        surrealdb_types::Value::String(item.into_string())
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "surrealdb")))]
impl TryFrom<surrealdb_types::Value> for CompactString {
    type Error = &'static str;

    fn try_from(value: surrealdb_types::Value) -> Result<Self, Self::Error> {
        match value {
            surrealdb_types::Value::String(v) => Ok(CompactString::new(v)),
            _ => Err("surrealdb value not string"),
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use surrealdb_types::{SurrealValue, ToSql, Value};

    use crate::CompactString;

    #[test]
    fn test_compact_string_to_value() {
        let compact = CompactString::from("test value");
        let value: Value = compact.into();

        assert!(value.is_string());
        assert_eq!(value.as_string(), Some(&"test value".to_string()));
    }

    #[test]
    fn test_value_to_compact_string() {
        let value = surrealdb_types::Value::String("test string value".to_string());
        let compact = CompactString::try_from(value).unwrap();

        assert_eq!("test string value".to_string(), compact);
    }

    #[test]
    fn test_derive() {
        #[derive(surrealdb_types::SurrealValue)]
        pub struct TestStruct {
            compact: CompactString,
        }

        let str = TestStruct {
            compact: CompactString::new("Test CompactString"),
        };

        let value = str.into_value();
        let sql = value.to_sql();

        assert!(!sql.is_empty());
    }
}
