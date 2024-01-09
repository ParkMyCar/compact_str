#![cfg(all(test, not(miri)))]

use std::hint::black_box;

use compact_str::{
    format_compact,
    CompactString,
};
use sqlx::{
    query,
    query_as,
    query_with,
    Acquire,
    Arguments,
    Executor,
    Row,
};
use tempfile::tempdir;

const TITLE: CompactString = CompactString::const_new("Say hello!");

macro_rules! test_body {
    ($($test_name:ident($DbPool:path, $DbArguments:path, $compile_only:literal);)*) => {$(
        #[tokio::test]
        async fn $test_name() -> Result<(), Box<dyn std::error::Error>> {
            if black_box($compile_only) {
                return Ok(());
            }

            // create and open a temporary database
            let tempdir = tempdir()?;
            let db_path = tempdir.path().join("compact_str.sqlite3");
            std::fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&db_path)?;

            let db_path = format_compact!("file://{}", db_path.to_str().unwrap());
            let pool = <$DbPool>::connect(&db_path).await?;
            let mut conn = pool.acquire().await?;
            conn.execute(query(include_str!("schema.sql"))).await?;

            // the database should be empty
            let mut transaction = conn.begin().await?;
            let q = query_as::<_, (i64, CompactString, bool)>(
                "SELECT * FROM todos WHERE TRUE LIMIT 10"
            );
            let rows = transaction.fetch_all(q).await?;
            assert!(rows.is_empty());
            transaction.commit().await?;

            // insert a new todo
            let mut transaction = conn.begin().await?;
            let mut args = <$DbArguments>::default();
            args.add(TITLE);
            args.add(false);
            let q = query_with("INSERT INTO todos (title, done) VALUES ($1, $2)", args);
            transaction.execute(q).await?;
            transaction.commit().await?;

            // check that the entry was inserted
            let mut transaction = conn.begin().await?;
            let q = query_as::<_, (i64, CompactString, bool)>(
                "SELECT id, title, done FROM todos WHERE TRUE LIMIT 10",
            );
            let rows = transaction.fetch_all(q).await?;
            assert_eq!(rows.len(), 1);
            let row = rows.first().unwrap();
            let id: i64 = row.try_get(0)?;
            let title: CompactString = row.try_get(1)?;
            let done: bool = row.try_get(2)?;
            assert_ne!(id, 0);
            assert!(!title.is_heap_allocated());
            assert_eq!(title, TITLE);
            assert!(!done);
            transaction.commit().await?;

            // say "hello" and and mark as done
            let mut transaction = conn.begin().await?;
            println!("Hello!");
            let mut args = <$DbArguments>::default();
            args.add(true);
            args.add(id);
            let q = query_with("UPDATE todos SET done = $1 WHERE id = $2", args);
            transaction.execute(q).await?;
            transaction.commit().await?;

            // check that the entry was updated
            let mut transaction = conn.begin().await?;
            let q = query_as::<_, (i64, CompactString, bool)>(
                "SELECT id, title, done FROM todos WHERE TRUE LIMIT 10",
            );
            let rows = transaction.fetch_all(q).await?;
            assert_eq!(rows.len(), 1);
            let row = rows.first().unwrap();
            let id: i64 = row.try_get(0)?;
            let title: CompactString = row.try_get(1)?;
            let done: bool = row.try_get(2)?;
            assert_ne!(id, 0);
            assert!(!title.is_heap_allocated());
            assert_eq!(title, TITLE);
            assert!(done);
            transaction.commit().await?;

            // we are done, delete our todo
            let mut transaction = conn.begin().await?;
            let mut args = <$DbArguments>::default();
            args.add(TITLE);
            let q = query_with("DELETE FROM todos WHERE title = $1", args);
            transaction.execute(q).await?;
            transaction.commit().await?;

            // the database should be empty
            let mut transaction = conn.begin().await?;
            let q = query_as::<_, (i64, CompactString, bool)>(
                "SELECT id, title, done FROM todos WHERE TRUE LIMIT 10"
            );
            let rows = transaction.fetch_all(q).await?;
            assert!(rows.is_empty());
            transaction.commit().await?;

            Ok(())
        }
    )*};
}

test_body! {
    sqlx_mysql(sqlx::mysql::MySqlPool, sqlx::mysql::MySqlArguments, true);
    sqlx_postgres(sqlx::postgres::PgPool, sqlx::postgres::PgArguments, true);
    sqlx_sqlite(sqlx::sqlite::SqlitePool, sqlx::sqlite::SqliteArguments, false);
}
