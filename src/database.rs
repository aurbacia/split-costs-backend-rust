use deadpool_diesel::postgres::{Manager, Pool, Runtime};
use std::env;

pub fn establish_connection() -> Pool {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool_size = env::var("MAX_DB_POOL_SIZE")
        .expect("MAX_DB_POOL_SIZE must be set")
        .parse::<usize>()
        .expect("MAX_DB_POOL_SIZE must be unsigned number");

    let manager = Manager::new(database_url, Runtime::Tokio1);

    return Pool::builder(manager).max_size(pool_size).build().unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::sql_types::Text;
    use diesel::{IntoSql, RunQueryDsl, select};

    #[tokio::test]
    #[serial_test::serial]
    async fn can_create_pool_with_correct_env() {
        dotenvy::dotenv_override().ok();
        establish_connection();
    }

    #[tokio::test]
    #[serial_test::serial]
    #[should_panic(expected = "DATABASE_URL must be set")]
    async fn cant_create_pool_without_db_url_env() {
        dotenvy::dotenv_override().ok();
        unsafe {
            env::remove_var("DATABASE_URL");
        }

        establish_connection();
    }

    #[tokio::test]
    #[serial_test::serial]
    #[should_panic(expected = "MAX_DB_POOL_SIZE must be set")]
    async fn cant_create_pool_without_pool_size_env() {
        dotenvy::dotenv_override().ok();
        unsafe {
            env::remove_var("MAX_DB_POOL_SIZE");
        }

        establish_connection();
    }

    #[tokio::test]
    #[serial_test::serial]
    #[should_panic(expected = "MAX_DB_POOL_SIZE must be unsigned number")]
    async fn cant_create_pool_with_pool_size_env_string() {
        dotenvy::dotenv_override().ok();
        unsafe {
            env::set_var("MAX_DB_POOL_SIZE", "not-number");
        }

        establish_connection();
    }

    #[tokio::test]
    #[serial_test::serial]
    #[should_panic(expected = "MAX_DB_POOL_SIZE must be unsigned number")]
    async fn cant_create_pool_with_pool_size_env_signed_int() {
        dotenvy::dotenv_override().ok();
        unsafe {
            env::set_var("MAX_DB_POOL_SIZE", "-500");
        }

        establish_connection();
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn can_use_connection_from_pool() {
        dotenvy::dotenv_override().ok();
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
    #[serial_test::serial]
    async fn can_get_a_lot_of_connections_from_pool() {
        dotenvy::dotenv_override().ok();
        let pool = std::sync::Arc::new(establish_connection());
        let mut set = tokio::task::JoinSet::new();
        let pool_size = env::var("MAX_DB_POOL_SIZE")
            .unwrap()
            .parse::<usize>()
            .unwrap()
            * 20;

        for _ in 0..pool_size {
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

        assert_eq!(set.len(), pool_size);
        for r in set.join_all().await {
            assert_eq!(r, "Test");
        }
    }
}
