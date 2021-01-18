use async_trait::async_trait;

use crate::Params;

#[derive(Clone, Debug)]
pub enum JoinType {
    Inner,
    Left,
    Right,
}

#[derive(Clone, Debug)]
pub enum Ordering {
    Ascending,
    Descending,
}

impl Ordering {
    pub fn to_string(&self) -> String {
        use Ordering::*;

        match self {
            Ascending => "ASC",
            Descending => "DESC",
        }
        .to_string()
    }
}

#[derive(Clone, Debug)]
pub struct QueryBuilder<V> {
    selects: Vec<String>,
    from: Option<String>,
    wheres: Vec<String>,
    limit: Option<i32>,
    joins: Vec<(JoinType, String, String, String)>,
    groups: Vec<String>,
    orders: Vec<(String, Ordering)>,
    binds: Vec<(String, V)>,
}

impl<V> QueryBuilder<V> {
    pub fn new() -> QueryBuilder<V> {
        QueryBuilder {
            selects: vec![],
            from: None,
            wheres: Vec::new(),
            limit: None,
            joins: vec![],
            groups: vec![],
            orders: vec![],
            binds: vec![],
        }
    }

    pub fn table(mut self, table_name: impl Into<String>) -> QueryBuilder<V> {
        self.from = Some(table_name.into());

        self
    }

    pub fn selects(mut self, selects: Vec<impl Into<String>>) -> QueryBuilder<V> {
        self.selects = selects.into_iter().map(|v| v.into()).collect::<Vec<_>>();

        self
    }

    pub fn append_selects(mut self, selects: Vec<impl Into<String>>) -> QueryBuilder<V> {
        self.selects
            .append(&mut selects.into_iter().map(|v| v.into()).collect::<Vec<_>>());

        self
    }

    pub fn wheres<S: Into<String>>(
        mut self,
        (cond, params): (Vec<S>, Vec<(String, V)>),
    ) -> QueryBuilder<V> {
        self.wheres
            .append(&mut cond.into_iter().map(|v| v.into()).collect::<Vec<_>>());

        self
    }

    pub fn filter(mut self, cond: impl Into<String>) -> QueryBuilder<V> {
        self.wheres.push(cond.into());

        self
    }

    pub fn limit(mut self, n: i32) -> QueryBuilder<V> {
        self.limit = Some(n);

        self
    }

    pub fn order_by(mut self, column_name: impl Into<String>, ordering: Ordering) -> Self {
        self.orders.push((column_name.into(), ordering));

        self
    }

    pub fn inner_join(
        mut self,
        target_table: impl Into<String>,
        on_fields: (impl Into<String>, impl Into<String>),
    ) -> QueryBuilder<V> {
        self.joins.push((
            JoinType::Inner,
            target_table.into(),
            on_fields.0.into(),
            on_fields.1.into(),
        ));

        self
    }

    pub fn left_join(
        mut self,
        target_table: impl Into<String>,
        on_fields: (impl Into<String>, impl Into<String>),
    ) -> QueryBuilder<V> {
        self.joins.push((
            JoinType::Left,
            target_table.into(),
            on_fields.0.into(),
            on_fields.1.into(),
        ));

        self
    }

    pub fn right_join(
        mut self,
        target_table: impl Into<String>,
        on_fields: (impl Into<String>, impl Into<String>),
    ) -> QueryBuilder<V> {
        self.joins.push((
            JoinType::Right,
            target_table.into(),
            on_fields.0.into(),
            on_fields.1.into(),
        ));

        self
    }

    pub fn group_by<S: Into<String>>(mut self, fields: Vec<S>) -> QueryBuilder<V> {
        self.groups
            .append(&mut fields.into_iter().map(|v| v.into()).collect::<Vec<_>>());

        self
    }

    pub fn build(self) -> (String, Params<V>) {
        let table = self.from.unwrap();
        let from = format!("FROM {}", table.clone());
        let where_clause = format!("WHERE {}", self.wheres.as_slice().join(" AND "));
        let limit_clause = self
            .limit
            .map(|i| format!("LIMIT {}", i))
            .unwrap_or("".to_string());

        (
            [
                // SELECT clause
                format!(
                    "SELECT {}",
                    if self.selects.is_empty() {
                        "*".to_string()
                    } else {
                        self.selects.as_slice().join(", ")
                    }
                ),
                from,
                // JOIN clause
                self.joins
                    .into_iter()
                    .map(|(jt, another_table, lhs, rhs)| {
                        format!(
                            "{} JOIN {} ON {}.{} = {}.{}",
                            match jt {
                                JoinType::Inner => "INNER",
                                JoinType::Left => "LEFT",
                                JoinType::Right => "RIGHT",
                            },
                            another_table,
                            table,
                            lhs,
                            another_table,
                            rhs
                        )
                    })
                    .collect::<Vec<_>>()
                    .as_slice()
                    .join(" "),
                // WHERE clause
                if !self.wheres.is_empty() {
                    where_clause
                } else {
                    String::new()
                },
                // GROUP BY clause
                if !self.groups.is_empty() {
                    format!("GROUP BY {}", self.groups.as_slice().join(", "))
                } else {
                    String::new()
                },
                // ORDER BY clause
                if !self.orders.is_empty() {
                    format!(
                        "ORDER BY {}",
                        self.orders
                            .into_iter()
                            .map(|(k, o)| format!("{} {}", k, o.to_string()))
                            .collect::<Vec<_>>()
                            .as_slice()
                            .join(", ")
                    )
                } else {
                    String::new()
                },
                // LIMIT clause
                limit_clause,
            ]
            .iter()
            .filter(|s| s.len() != 0)
            .cloned()
            .collect::<Vec<_>>()
            .as_slice()
            .join(" "),
            Params(self.binds),
        )
    }

    pub async fn load<R: QueryExecutor<T, E, ValueType = V>, T, E>(
        self,
        executor: &R,
    ) -> Result<Vec<T>, E> {
        executor.load(self).await
    }

    pub async fn first<R: QueryExecutor<T, E, ValueType = V>, T, E>(
        self,
        executor: &R,
    ) -> Result<T, E> {
        executor.first(self).await
    }

    pub async fn save<R: QueryExecutor<T, E, ValueType = V>, T, E>(
        self,
        executor: &R,
        data: T,
    ) -> Result<(), E> {
        executor.save(self, data).await
    }

    pub async fn save_all<R: QueryExecutor<T, E, ValueType = V>, T, E>(
        self,
        executor: &R,
        data: Vec<T>,
    ) -> Result<(), E> {
        executor.save_all(self, data).await
    }
}

#[async_trait]
pub trait QueryExecutor<T, E> {
    type ValueType;

    async fn load(&self, builder: QueryBuilder<Self::ValueType>) -> Result<Vec<T>, E>;
    async fn first(&self, builder: QueryBuilder<Self::ValueType>) -> Result<T, E>;
    async fn save(&self, builder: QueryBuilder<Self::ValueType>, data: T) -> Result<(), E>;
    async fn save_all(&self, builder: QueryBuilder<Self::ValueType>, data: Vec<T>)
        -> Result<(), E>;
}

#[test]
fn query_with_build() {
    assert_eq!(
        QueryBuilder::<()>::new().table("foo").build().0,
        "SELECT * FROM foo"
    );
    assert_eq!(
        QueryBuilder::<()>::new()
            .table("foo")
            .selects(vec!["a", "b", "c"])
            .build()
            .0,
        "SELECT a, b, c FROM foo"
    );
    assert_eq!(
        QueryBuilder::<()>::new()
            .table("foo")
            .wheres((vec!["bar = 10"], vec![]))
            .build()
            .0,
        "SELECT * FROM foo WHERE bar = 10"
    );
    assert_eq!(
        QueryBuilder::<()>::new()
            .table("foo")
            .wheres((vec!["bar = 10", "baz = 20"], vec![]))
            .limit(10)
            .build()
            .0,
        "SELECT * FROM foo WHERE bar = 10 AND baz = 20 LIMIT 10"
    );
    assert_eq!(
        QueryBuilder::<()>::new()
            .table("foo")
            .inner_join("bar", ("baz", "quux"))
            .build()
            .0,
        "SELECT * FROM foo INNER JOIN bar ON foo.baz = bar.quux"
    );
    assert_eq!(
        QueryBuilder::<()>::new()
            .table("foo")
            .inner_join("bar", ("baz", "quux"))
            .left_join("bar2", ("baz2", "quux2"))
            .build().0,
        "SELECT * FROM foo INNER JOIN bar ON foo.baz = bar.quux LEFT JOIN bar2 ON foo.baz2 = bar2.quux2"
    );
    assert_eq!(
        QueryBuilder::<()>::new()
            .table("foo")
            .group_by(vec!["a", "b", "c"])
            .build()
            .0,
        "SELECT * FROM foo GROUP BY a, b, c"
    );
    assert_eq!(
        QueryBuilder::<()>::new()
            .table("foo")
            .order_by("piyo", Ordering::Ascending)
            .order_by("nyan", Ordering::Descending)
            .build()
            .0,
        "SELECT * FROM foo ORDER BY piyo ASC, nyan DESC"
    );
}
