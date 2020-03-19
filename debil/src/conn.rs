use crate::{QueryBuilder, SQLMapper, SQLTable};
use async_trait::async_trait;
use std::collections::HashMap;

pub struct Params<ValueType>(pub Vec<(String, ValueType)>);

impl<V> Params<V> {
    pub fn new() -> Params<V> {
        Params(Vec::new())
    }
}

pub trait HasNotFound {
    fn not_found() -> Self;
}

#[async_trait]
pub trait SQLConn<V: 'static + Sync + Send> {
    type Error: HasNotFound;

    async fn sql_exec(&mut self, query: String, params: Params<V>) -> Result<u64, Self::Error>;

    async fn sql_query<T: SQLMapper<ValueType = V> + Sync + Send>(
        &mut self,
        query: String,
        params: Params<V>,
    ) -> Result<Vec<T>, Self::Error>;

    async fn sql_batch_exec(
        &mut self,
        query: String,
        params: Vec<Params<V>>,
    ) -> Result<(), Self::Error>;

    async fn create_table<T: SQLTable<ValueType = V> + Sync + Send>(
        &mut self,
    ) -> Result<(), Self::Error> {
        self.sql_exec(
            SQLTable::create_table_query(std::marker::PhantomData::<T>),
            Params::<V>(Vec::new()),
        )
        .await?;

        Ok(())
    }

    async fn create<T: SQLTable<ValueType = V> + Sync + Send>(
        &mut self,
        data: T,
    ) -> Result<u64, Self::Error> {
        let (query, ps) = data.insert_query_with_params();

        self.sql_exec(query, Params::<V>(ps)).await
    }

    async fn save<T: SQLTable<ValueType = V> + Sync + Send + Clone>(
        &mut self,
        data: T,
    ) -> Result<u64, Self::Error> {
        let (query, ps) = data.clone().update_query_with_params();
        let affected_rows = self.sql_exec(query, Params::<V>(ps)).await?;

        if affected_rows == 0 {
            self.create(data).await
        } else {
            Ok(affected_rows)
        }
    }

    async fn load_with2<T: SQLTable, U: SQLMapper<ValueType = V> + Sync + Send>(
        &mut self,
        builder: QueryBuilder,
    ) -> Result<Vec<U>, Self::Error> {
        let schema = SQLTable::schema_of(std::marker::PhantomData::<T>);
        let table_name = SQLTable::table_name(std::marker::PhantomData::<T>);
        let query = builder
            .table(table_name.clone())
            .append_selects(
                schema
                    .iter()
                    .map(|(k, _, _)| format!("{}.{}", table_name, k))
                    .collect::<Vec<_>>(),
            )
            .build();
        self.sql_query::<U>(query, Params(Vec::new())).await
    }

    async fn load_with<T: SQLTable<ValueType = V> + Sync + Send>(
        &mut self,
        builder: QueryBuilder,
    ) -> Result<Vec<T>, Self::Error> {
        self.load_with2::<T, T>(builder).await
    }

    async fn first_with<T: SQLTable<ValueType = V> + Sync + Send>(
        &mut self,
        builder: QueryBuilder,
    ) -> Result<T, Self::Error> {
        let schema = SQLTable::schema_of(std::marker::PhantomData::<T>);
        let table_name = SQLTable::table_name(std::marker::PhantomData::<T>);
        let query = builder
            .table(table_name.clone())
            .append_selects(
                schema
                    .iter()
                    .map(|(k, _, _)| format!("{}.{}", table_name, k))
                    .collect::<Vec<_>>(),
            )
            .limit(1)
            .build();

        self.sql_query::<T>(query, Params(Vec::new()))
            .await
            .and_then(|mut vs| vs.pop().ok_or(HasNotFound::not_found()))
    }

    async fn load<T: SQLTable<ValueType = V> + Sync + Send>(
        &mut self,
    ) -> Result<Vec<T>, Self::Error> {
        self.load_with(QueryBuilder::new()).await
    }

    async fn first<T: SQLTable<ValueType = V> + Sync + Send>(&mut self) -> Result<T, Self::Error> {
        self.first_with(QueryBuilder::new()).await
    }
}
