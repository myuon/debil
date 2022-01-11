pub fn create_index(table_name: &str, index_name: &str, columns: &[&str]) -> String {
    format!(
        "CREATE INDEX IF NOT EXISTS {} ON {} ({});",
        index_name,
        table_name,
        columns.join(", ")
    )
}
