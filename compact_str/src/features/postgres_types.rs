use postgres_types::{private::BytesMut, to_sql_checked, Format, FromSql, IsNull, ToSql, Type};
use std::{boxed::Box, error::Error};

use crate::CompactString;

#[cfg(feature = "postgres-types")]
impl<'a> FromSql<'a> for CompactString {
    fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
        <&str as FromSql>::from_sql(ty, raw).map(CompactString::from)
    }

    fn accepts(ty: &Type) -> bool {
        <&str as FromSql>::accepts(ty)
    }

    fn from_sql_null(ty: &Type) -> Result<Self, Box<dyn Error + Sync + Send>> {
        <&str as FromSql>::from_sql_null(ty).map(CompactString::from)
    }

    fn from_sql_nullable(
        ty: &Type,
        raw: Option<&'a [u8]>,
    ) -> Result<Self, Box<dyn Error + Sync + Send>> {
        <&str as FromSql>::from_sql_nullable(ty, raw).map(CompactString::from)
    }
}

#[cfg(feature = "postgres-types")]
impl ToSql for CompactString {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        <&str as ToSql>::to_sql(&&**self, ty, out)
    }

    fn accepts(ty: &Type) -> bool {
        <&str as ToSql>::accepts(ty)
    }

    to_sql_checked!();

    fn encode_format(&self, ty: &Type) -> Format {
        <&str as ToSql>::encode_format(&&**self, ty)
    }
}
