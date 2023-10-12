use compact_str::CompactString;
use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::todos)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Todo {
    pub id: i32,
    pub title: CompactString,
    pub done: bool,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::todos)]
pub struct NewTodo {
    pub title: CompactString,
    pub done: bool,
}
