diesel::table! {
    todos (id) {
        id -> Integer,
        title -> Text,
        done -> Bool,
    }
}
