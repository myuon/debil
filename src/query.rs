pub struct QueryBuilder {
    selects: Vec<String>,
    from: Option<String>,
    wheres: Vec<String>,
    limit: Option<i32>,
}

impl QueryBuilder {
    pub fn new() -> QueryBuilder {
        QueryBuilder {
            selects: vec![],
            from: None,
            wheres: Vec::new(),
            limit: None,
        }
    }

    pub fn table(mut self, table_name: impl Into<String>) -> QueryBuilder {
        self.from = Some(table_name.into());

        self
    }

    pub fn selects(mut self, selects: Vec<impl Into<String>>) -> QueryBuilder {
        self.selects = selects.into_iter().map(|v| v.into()).collect::<Vec<_>>();

        self
    }

    pub fn wheres(mut self, cond: impl Into<String>) -> QueryBuilder {
        self.wheres.push(cond.into());

        self
    }

    pub fn limit(mut self, n: i32) -> QueryBuilder {
        self.limit = Some(n);

        self
    }

    pub fn build(&self) -> String {
        let from = format!("FROM {}", self.from.clone().unwrap());
        let where_clause = format!("WHERE {}", self.wheres.as_slice().join(" AND "));
        let limit_clause = self
            .limit
            .map(|i| format!("LIMIT {}", i))
            .unwrap_or("".to_string());

        [
            format!(
                "SELECT {}",
                if self.selects.is_empty() {
                    "*".to_string()
                } else {
                    self.selects.as_slice().join(", ")
                }
            )
            .as_str(),
            from.as_str(),
            if !self.wheres.is_empty() {
                where_clause.as_str()
            } else {
                ""
            },
            limit_clause.as_str(),
        ]
        .into_iter()
        .filter(|s| s.len() != 0)
        .cloned()
        .collect::<Vec<_>>()
        .as_slice()
        .join(" ")
    }
}

#[test]
fn query_with_build() {
    assert_eq!(
        QueryBuilder::new().table("foo").build(),
        "SELECT * FROM foo"
    );
    assert_eq!(
        QueryBuilder::new()
            .table("foo")
            .selects(vec!["a", "b", "c"])
            .build(),
        "SELECT a, b, c FROM foo"
    );
    assert_eq!(
        QueryBuilder::new().table("foo").wheres("bar = 10").build(),
        "SELECT * FROM foo WHERE bar = 10"
    );
    assert_eq!(
        QueryBuilder::new()
            .table("foo")
            .wheres("bar = 10")
            .limit(10)
            .build(),
        "SELECT * FROM foo WHERE bar = 10 LIMIT 10"
    );
}
