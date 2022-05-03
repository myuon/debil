#[cfg(feature = "mysql")]
mod tests {
    use debil::mysql::*;
    use debil::*;
    use mysql_async::OptsBuilder;

    #[derive(Table, PartialEq, Debug, Clone)]
    #[sql(table_name = "user", primary_key = "user_id")]
    struct User {
        #[sql(size = 50)]
        user_id: String,
        #[sql(size = 50, unqiue = true)]
        name: String,
        #[sql(size = 256)]
        email: String,
        age: i32,
    }

    #[derive(Table, Clone)]
    #[sql(table_name = "user_item_relation", primary_key = "user_id, item_id")]
    struct UserItem {
        #[sql(size = 50)]
        user_id: String,
        #[sql(size = 50)]
        item_id: String,
    }

    #[derive(Debug, PartialEq)]
    struct JoinedUserItemsView {
        user: User,
        item_id: Option<String>,
    }

    impl SqlMapper for JoinedUserItemsView {
        type ValueType = MySQLValue;

        fn map_from_sql(
            h: std::collections::HashMap<String, Self::ValueType>,
        ) -> JoinedUserItemsView {
            let item_id = h["item_id"].clone();
            let user = map_from_sql::<User>(h);

            JoinedUserItemsView {
                user,
                item_id: <Self::ValueType>::deserialize(item_id),
            }
        }
    }

    async fn setup(conn: &mut DebilConn) -> Result<(), Error> {
        // drop table
        conn.drop_table::<User>().await?;
        conn.drop_table::<UserItem>().await?;

        // create
        conn.create_table::<User>().await?;
        conn.create_table::<UserItem>().await?;

        Ok(())
    }

    // for sequential testing, we use only one function to test
    #[tokio::test]
    async fn it_should_create_and_select() -> Result<(), Error> {
        let raw_conn = mysql_async::Conn::new(
            OptsBuilder::default()
                .ip_or_hostname("127.0.0.1")
                .user(Some("root"))
                .pass(Some("password"))
                .db_name(Some("db"))
                .prefer_socket(Some(false))
                .pool_opts(Some(mysql_async::PoolOpts::default().with_constraints(
                    mysql_async::PoolConstraints::new(1, 1).unwrap(),
                )))
                .clone(),
        )
        .await?;
        let mut conn = DebilConn::from_conn(raw_conn);
        setup(&mut conn).await?;

        let user1 = User {
            user_id: "user-123456".to_string(),
            name: "foo".to_string(),
            email: "dddd@example.com".to_string(),
            age: 20,
        };
        let user2 = User {
            user_id: "user-456789".to_string(),
            name: "bar".to_string(),
            email: "quux@example.com".to_string(),
            age: 55,
        };
        conn.save::<User>(user1.clone()).await?;
        conn.save::<User>(user2.clone()).await?;

        conn.create_all::<User>(vec![
            User {
                user_id: "_a".to_string(),
                name: "".to_string(),
                email: "".to_string(),
                age: 0,
            },
            User {
                user_id: "_b".to_string(),
                name: "".to_string(),
                email: "".to_string(),
                age: 0,
            },
        ])
        .await?;

        let result = conn.load::<User>(QueryBuilder::new()).await?;
        assert_eq!(result.len(), 4);
        assert_eq!(result[0..2].to_vec(), vec![user1, user2]);

        // save for create/update
        let mut user3 = User {
            user_id: "user-savetest".to_string(),
            name: "foo".to_string(),
            email: "dddd@example.com".to_string(),
            age: 20,
        };
        conn.save::<User>(user3.clone()).await?;

        let user3_result = conn
            .first::<User>(QueryBuilder::new().filter(format!(
                "{}.user_id = '{}'",
                table_name::<User>(),
                "user-savetest"
            )))
            .await?;
        assert_eq!(user3_result.age, 20);

        user3.age = 21;
        conn.save::<User>(user3.clone()).await?;

        let user3_result = conn
            .first::<User>(QueryBuilder::new().filter(format!(
                "{}.user_id = '{}'",
                table_name::<User>(),
                "user-savetest"
            )))
            .await?;
        assert_eq!(user3_result.age, 21);

        // join query
        let user_id = "user-join-and-load".to_string();
        let user = User {
            user_id: user_id.clone(),
            name: "foo".to_string(),
            email: "dddd@example.com".to_string(),
            age: 20,
        };
        conn.save(user.clone()).await?;
        conn.create_all(vec![
            UserItem {
                user_id: user_id.clone(),
                item_id: "item-abcd".to_string(),
            },
            UserItem {
                user_id: user_id.clone(),
                item_id: "item-defg".to_string(),
            },
            UserItem {
                user_id: user_id.clone(),
                item_id: "item-pqrs".to_string(),
            },
        ])
        .await?;

        let j = conn
            .load2::<User, JoinedUserItemsView>(
                QueryBuilder::new()
                    .left_join(table_name::<UserItem>(), ("user_id", "user_id"))
                    .filter(format!("{}.user_id = '{}'", table_name::<User>(), user_id))
                    .append_selects(vec![format!("{}.item_id", table_name::<UserItem>())]),
            )
            .await?;

        assert_eq!(
            j,
            vec![
                JoinedUserItemsView {
                    user: user.clone(),
                    item_id: Some("item-abcd".to_string()),
                },
                JoinedUserItemsView {
                    user: user.clone(),
                    item_id: Some("item-defg".to_string()),
                },
                JoinedUserItemsView {
                    user: user.clone(),
                    item_id: Some("item-pqrs".to_string()),
                }
            ]
        );

        // check thread safety
        async fn conn_load(mut conn: DebilConn) {
            conn.load::<User>(QueryBuilder::new()).await.unwrap();
        }
        tokio::spawn(conn_load(conn)).await.unwrap();

        Ok(())
    }
}
