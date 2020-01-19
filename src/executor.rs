use crate::{QueryBuilder, SQLMapper, SQLTable};
use async_trait::async_trait;
use std::collections::HashMap;

pub struct Params<ValueType>(pub Vec<(String, ValueType)>);

pub trait HasNotFound {
    fn not_found() -> Self;
}

#[async_trait]
pub trait QueryExecutor {
    type Error: HasNotFound;

    async fn sql_exec<V: Sync + Send>(
        &mut self,
        query: String,
        params: Params<V>,
    ) -> Result<u64, Self::Error>;

    async fn sql_query<T: SQLMapper<ValueType = V> + Sync + Send, V: Sync + Send>(
        &mut self,
        query: String,
        params: Params<V>,
    ) -> Result<Vec<T>, Self::Error>;

    async fn sql_batch_exec(
        &mut self,
        query: String,
        params: HashMap<String, String>,
    ) -> Result<(), Self::Error>;

    async fn create_table<T: SQLTable<ValueType = V> + Sync + Send, V: Sync + Send>(
        &mut self,
    ) -> Result<(), Self::Error> {
        self.sql_exec(
            SQLTable::create_table_query(std::marker::PhantomData::<T>),
            Params::<V>(Vec::new()),
        )
        .await?;

        Ok(())
    }

    // FIXME: update
    async fn save<T: SQLTable<ValueType = V> + Sync + Send, V: Sync + Send>(
        &mut self,
        data: T,
    ) -> Result<(), Self::Error> {
        let (query, ps) = data.save_query_with_params();
        self.sql_exec(query, Params::<V>(ps)).await?;

        Ok(())
    }

    async fn load_with2<T: SQLTable, U: SQLMapper<ValueType = V> + Sync + Send, V: Sync + Send>(
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
        self.sql_query::<U, V>(query, Params(Vec::new())).await
    }

    async fn load_with<T: SQLTable<ValueType = V> + Sync + Send, V: Sync + Send>(
        &mut self,
        builder: QueryBuilder,
    ) -> Result<Vec<T>, Self::Error> {
        self.load_with2::<T, T, V>(builder).await
    }

    async fn first_with<T: SQLTable<ValueType = V> + Sync + Send, V: Sync + Send>(
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

        self.sql_query::<T, V>(query, Params(Vec::new()))
            .await
            .and_then(|mut vs| vs.pop().ok_or(HasNotFound::not_found()))
    }

    async fn load<T: SQLTable<ValueType = V> + Sync + Send, V: Sync + Send>(
        &mut self,
    ) -> Result<Vec<T>, Self::Error> {
        self.load_with(QueryBuilder::new()).await
    }

    async fn first<T: SQLTable<ValueType = V> + Sync + Send, V: Sync + Send>(
        &mut self,
    ) -> Result<T, Self::Error> {
        self.first_with(QueryBuilder::new()).await
    }
}
