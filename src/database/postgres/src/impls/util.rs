use diesel::sql_types::{Nullable, Text};

diesel::define_sql_function!(fn coalesce(x: Nullable<Text>, y: Text) -> Text);
diesel::define_sql_function!(fn nullif(x: Text, y: Text) -> Nullable<Text>);
diesel::define_sql_function!(fn lower(x: Text) -> Text);
