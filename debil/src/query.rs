pub fn create_index(table_name: &str, index_name: &str, columns: &[&str]) -> String {
    format!(
        "CREATE INDEX IF NOT EXISTS {} ON {} ({});",
        index_name,
        table_name,
        columns.join(", ")
    )
}

#[allow(unused_variables)]
pub fn drop_index(table_name: &str, index_name: &str) -> String {
    #[cfg(feature = "sqlite")]
    {
        format!("DROP INDEX IF EXISTS {}", index_name)
    }
    #[cfg(not(feature = "sqlite"))]
    {
        format!("DROP INDEX IF EXISTS {} ON {};", index_name, table_name)
    }
}
