use deadpool_diesel::postgres::{Manager, Pool, Runtime};
use std::env;

pub fn establish_connection() -> Pool {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = Manager::new(database_url, Runtime::Tokio1);

    return Pool::builder(manager).max_size(8).build().unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::sql_types::Text;
    use diesel::{IntoSql, RunQueryDsl, select};

    #[tokio::test]
    async fn can_create_pool_with_correct_env() {
        dotenvy::dotenv().ok();
        establish_connection();
    }

    #[serial_test::serial]
    #[tokio::test]
    #[should_panic(expected = "DATABASE_URL must be set")]
    async fn cant_create_pool_without_env_value() {
        unsafe {
            env::remove_var("DATABASE_URL");
        }

        establish_connection();
    }

    #[tokio::test]
    async fn can_use_connection_from_pool() {
        dotenvy::dotenv().ok();
        let pool = establish_connection();
        let conn = pool.get().await.unwrap();
        let result = conn
            .interact(|conn| {
                let query = select("Test".into_sql::<Text>());
                query.get_result::<String>(conn)
            })
            .await
            .unwrap()
            .unwrap();

        assert_eq!(result, "Test")
    }

    #[tokio::test]
    async fn can_get_a_lot_of_connections_from_pool() {
        dotenvy::dotenv().ok();
        let pool = std::sync::Arc::new(establish_connection());
        let mut set = tokio::task::JoinSet::new();
        for _ in 0..500 {
            let cloned_pool = pool.clone();
            set.spawn(async move {
                let conn = cloned_pool.get().await.unwrap();

                conn.interact(|conn| {
                    let query = select("Test".into_sql::<Text>());
                    query.get_result::<String>(conn)
                })
                .await
                .unwrap()
                .unwrap()
            });
        }

        assert_eq!(set.len(), 500);
        for r in set.join_all().await {
            assert_eq!(r, "Test");
        }
    }
}
