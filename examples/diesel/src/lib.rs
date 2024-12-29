#![cfg(all(test, not(miri)))]

mod models;
mod schema;

use compact_str::{format_compact, CompactString};
use diesel::prelude::*;
use diesel::sql_query;
use tempfile::tempdir;

use crate::models::{NewTodo, Todo};
use crate::schema::todos;
use crate::schema::todos::dsl::*;

const TITLE: CompactString = CompactString::const_new("Say hello!");

#[test]
fn diesel_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    // create and open a temporary database
    let tempdir = tempdir()?;
    let db_path = tempdir.path().join("compact_str.sqlite3");
    let db_path = format_compact!("file://{}", db_path.to_str().unwrap());

    let mut conn = SqliteConnection::establish(&db_path)?;
    sql_query(include_str!("schema.sql")).execute(&mut conn)?;

    // the database should be empty
    let todo_list =
        conn.transaction(|conn| todos.limit(10).select(Todo::as_select()).load(conn))?;
    assert!(todo_list.is_empty());

    // insert a new todo
    conn.transaction(|conn| {
        let new_todo = NewTodo {
            title: TITLE,
            done: false,
        };
        diesel::insert_into(todos::table)
            .values(&new_todo)
            .execute(conn)
    })?;

    // check that the entry was inserted
    let todo_list =
        conn.transaction(|conn| todos.limit(10).select(Todo::as_select()).load(conn))?;
    assert_eq!(todo_list.len(), 1);
    let todo = todo_list.first().unwrap();
    assert!(!todo.title.is_heap_allocated());
    assert_eq!(todo.title, TITLE);
    assert!(!todo.done);

    // say "hello" and and mark as done
    conn.transaction(|conn| {
        print!("Hello!");
        diesel::update(todos.find(todo.id))
            .set(done.eq(true))
            .execute(conn)
    })?;

    // check that the entry was updated
    let todo_list =
        conn.transaction(|conn| todos.limit(10).select(Todo::as_select()).load(conn))?;
    assert_eq!(todo_list.len(), 1);
    let todo = todo_list.first().unwrap();
    assert!(!todo.title.is_heap_allocated());
    assert_eq!(todo.title, TITLE);
    assert!(todo.done);

    // we are done, delete our todo
    conn.transaction(|conn| diesel::delete(todos.filter(title.eq(TITLE))).execute(conn))?;

    // the database should be empty
    let todo_list =
        conn.transaction(|conn| todos.limit(10).select(Todo::as_select()).load(conn))?;
    assert!(todo_list.is_empty());

    Ok(())
}
