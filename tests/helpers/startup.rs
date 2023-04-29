use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use once_cell::sync::Lazy;

use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};
use sqlx::{ConnectOptions, Connection, PgConnection, Pool, Postgres, Row};
use std::convert::{TryFrom, TryInto};
use tracing_log::log;

use tokend::error::Error as TError;
use tokend::infra::config::{DatabaseRole, DatabaseSettings, Settings};
use tokend::infra::telemetry;

// Ensure that the `tracing` stack is only initialised once using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber =
            telemetry::get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        telemetry::init_subscriber(subscriber);
    } else {
        let subscriber =
            telemetry::get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        telemetry::init_subscriber(subscriber);
    };
});

pub async fn random_configuration() -> Settings {
    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(5)
        .map(char::from)
        .collect::<String>()
        .to_lowercase();

    let mut config =
        tokend::infra::config::get_configuration().expect("Failed to read configuration");
    config.web.port = 0;
    config.database.database_name =
        format!("test_{}_{}", chrono::Utc::now().format("%Y%m%d_%H%M%S"), rand_string);
    config
}

async fn create_database(settings: &DatabaseSettings) {
    let options = settings.without_db(&DatabaseRole::Root);
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect_with(options)
        .await
        .expect("Failed to create connection pool");

    let sqls = format!(
        "CREATE DATABASE {db_name} WITH
            TEMPLATE = template0
            ENCODING = 'UTF8'
            TABLESPACE = pg_default
            LC_COLLATE = 'en_US.utf8'
            LC_CTYPE = 'en_US.utf8'
            CONNECTION LIMIT = 255;
        ALTER DATABASE {db_name} OWNER TO {owner};
        GRANT CONNECT ON DATABASE {db_name} TO {owner};",
        db_name = settings.database_name,
        owner = settings
            .roles
            .get(&DatabaseRole::Migration)
            .expect("Migration role expected")
            .on_behalf_of
            .as_ref()
            .expect("Migration should act as database owner")
    );

    for sql in sqls.split(";") {
        sqlx::query(sql)
            .fetch_all(&pool)
            .await
            .expect(format!("Failed to execute statement: {}", sql).as_str());
    }
    tracing::info!("Database {} created", settings.database_name);
    pool.close().await
}

async fn create_schema(settings: &DatabaseSettings) {
    let options = settings.with_db(&DatabaseRole::Root);
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect_with(options)
        .await
        .expect("Failed to create connection pool");
    let mig = settings.roles.get(&DatabaseRole::Migration).unwrap();
    let app = settings.roles.get(&DatabaseRole::Application).unwrap();

    let sqls = format!("
        CREATE SCHEMA IF NOT EXISTS dev;
        ALTER SCHEMA dev OWNER TO {owner};
        ALTER ROLE {owner} IN DATABASE {dbname} SET search_path to dev, public;
        ALTER ROLE {app} IN DATABASE {dbname} SET search_path to dev, public;
        GRANT SELECT, UPDATE, INSERT, DELETE ON ALL TABLES    IN SCHEMA dev TO {app};
        GRANT SELECT, UPDATE, USAGE          ON ALL SEQUENCES IN SCHEMA dev TO {app};
        GRANT USAGE                          ON                  SCHEMA dev TO {app};
        ALTER DEFAULT PRIVILEGES FOR ROLE {owner} IN SCHEMA dev GRANT SELECT, UPDATE, INSERT, DELETE ON TABLES    TO {app};
        ALTER DEFAULT PRIVILEGES FOR ROLE {owner} IN SCHEMA dev GRANT SELECT, UPDATE, USAGE          ON SEQUENCES TO {app};
        ALTER ROLE {mig} IN DATABASE {dbname} SET search_path to dev, public;
        GRANT {owner} TO {mig};",
                       app = app.username,
                       mig = mig.username,
                       owner = mig.on_behalf_of.as_ref().unwrap(),
                       dbname = settings.database_name);

    let mut tx = pool.begin().await.unwrap();
    for sql in sqls.split(";") {
        sqlx::query(sql)
            .fetch_all(&mut tx)
            .await
            .expect("Failed to execute statement");
    }
    tx.commit().await.expect("Unable to commit transaction");
    tracing::info!("Schema created in {}", settings.database_name);
    pool.close().await
}

pub async fn spawn_app() {
    Lazy::force(&TRACING);
}

pub async fn spawn_db(settings: &DatabaseSettings) {
    Lazy::force(&TRACING);
    create_database(settings).await;
    create_schema(settings).await;
}

pub async fn migrate_db(settings: &DatabaseSettings) {
    let connect_options = settings.with_db(&DatabaseRole::Migration);
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect_with(connect_options)
        .await
        .expect("Failed to create connection pool");

    // open connection in a block
    // so that connection will be released by RAII and
    // pool won't stuck when trying to close it
    {
        let mut connection = pool.acquire().await.expect("Failed to acquire connection");

        if let Some(owner_role) = settings
            .roles
            .get(&DatabaseRole::Migration)
            .expect("Missing Migration Role")
            .on_behalf_of
            .as_ref()
        {
            let vec = sqlx::query(format!("SET ROLE {}", owner_role).as_str())
                .execute(&mut connection)
                .await
                .expect("Failed to set migration Role.");
        }

        sqlx::migrate!()
            .run(&mut connection)
            .await
            .expect("Unable to migrate database");
    }
    pool.close().await;
}


pub async fn drop_db(settings: &DatabaseSettings) {
    let options = settings.without_db(&DatabaseRole::Root);
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect_with(options)
        .await
        .expect("Failed to create connection pool");
    sqlx::query(format!("DROP DATABASE {};", settings.database_name).as_str())
        .execute(&pool)
        .await
        .expect("Failed to drop database.");
    pool.close().await
}
