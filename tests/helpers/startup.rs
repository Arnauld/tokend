use once_cell::sync::Lazy;

use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};
use sqlx::{ConnectOptions, PgConnection};
use std::convert::{TryFrom, TryInto};

use tokend::error::Error as TError;
use tokend::infra::config::{DatabaseRole, DatabaseSettings, Settings};
use tokend::infra::telemetry;

// Ensure that the `tracing` stack is only initialised once using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = telemetry::get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        telemetry::init_subscriber(subscriber);
    } else {
        let subscriber = telemetry::get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        telemetry::init_subscriber(subscriber);
    };
});

pub async fn random_configuration() -> Settings {
    let mut config = tokend::infra::config::get_configuration().expect("Failed to read configuration");
    config.web.port = 0;
    config.database.database_name = format!("test-{}", chrono::Utc::now().timestamp());
    config
}

pub async fn spawn_db(settings: &DatabaseSettings) {
    let options = settings.without_db(&DatabaseRole::Root);
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .expect("Failed to create connection pool");

    pool.acquire()
}