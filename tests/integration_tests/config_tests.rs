use crate::helpers::startup;
use sqlx::postgres::PgPoolOptions;
use tokend::infra::config::DatabaseRole;

#[tokio::test]
async fn database_url_is_not_empty() {
    dotenv::dotenv().expect("No .env defined");
    let db_url = std::env::var("DATABASE_URL").expect("Env var 'DATABASE_URL' not defined");
    assert!(!db_url.is_empty());
}

#[tokio::test]
async fn get_configuration_test_config() {
    std::env::set_var("APP_ENVIRONMENT", "local");
    std::env::set_var("APP_CONFIG_DIR", "./conf");
    let settings = tokend::infra::config::get_configuration().expect("Failed to load config");
    assert_eq!(settings.database.database_name, "tokend".to_string())
}

#[tokio::test]
async fn create_new_database_and_execute_migrations_and_drop_database() {
    std::env::set_var("APP_ENVIRONMENT", "local");
    std::env::set_var("APP_CONFIG_DIR", "./conf");
    let settings = startup::random_configuration().await;
    println!(">>> {:?}", &settings);

    startup::spawn_db(&settings.database).await;

    let connect_options = settings.database.with_db(&DatabaseRole::Migration);
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect_with(connect_options)
        .await
        .expect("Failed to create connection pool");
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Unable to migrate database");
    pool.close().await;

    startup::drop_db(&settings.database).await;
}
