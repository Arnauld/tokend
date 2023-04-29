use std::collections::HashMap;
use std::fmt::Formatter;

use config::ConfigError;
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Deserializer};
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use sqlx::ConnectOptions;
use tracing::metadata::LevelFilter;

use crate::error::Error as TError;

#[derive(serde::Deserialize, Debug, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub web: WebSettings,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct WebSettings {
    pub port: u16,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct DatabaseCredentials {
    pub username: String,
    pub password: Secret<String>,
    pub on_behalf_of: Option<String>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum DatabaseRole {
    Root,
    Application,
    Migration,
}

impl std::fmt::Display for DatabaseRole {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseRole::Root => write!(f, "Root"),
            DatabaseRole::Application => write!(f, "Application"),
            DatabaseRole::Migration => write!(f, "Migration"),
        }
    }
}

impl<'de> Deserialize<'de> for DatabaseRole {
    fn deserialize<D>(deserializer: D) -> Result<DatabaseRole, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "root" => Ok(DatabaseRole::Root),
            "application" => Ok(DatabaseRole::Application),
            "migration" => Ok(DatabaseRole::Migration),
            s => Err(serde::de::Error::custom(format!(
                "invalid value for DatabaseRole: {s}"
            ))),
        }
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct DatabaseSettings {
    pub host: String,
    pub port: u16,
    pub roles: HashMap<DatabaseRole, DatabaseCredentials>,
    pub database_name: String,
    pub require_ssl: Option<bool>,
}

impl DatabaseSettings {
    pub fn without_db(&self, role: &DatabaseRole) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl.unwrap_or(false) {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };
        let creds = self
            .roles
            .get(role)
            .expect(format!("Missing role credentials {role}").as_str());
        PgConnectOptions::new()
            .host(&self.host)
            .username(creds.username.as_str())
            .password(creds.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
    }

    pub fn with_db(&self, role: &DatabaseRole) -> PgConnectOptions {
        let mut options = self.without_db(role).database(&self.database_name);
        options.log_statements(tracing_log::log::LevelFilter::Trace);
        options
    }
}

fn check_settings(environment: &Environment, settings: &Settings) {
    if environment == &Environment::Prod {
        if settings.database.roles.contains_key(&DatabaseRole::Root) {
            panic!("Root role should not appear in production environment")
        }
    }
}

pub fn get_configuration() -> Result<Settings, TError> {
    let config_dir = std::env::var("APP_CONFIG_DIR").expect("'APP_CONFIG_DIR' not defined");
    let config_dir = std::path::PathBuf::from(config_dir);

    // Detect the running environment.
    // Default to `local` if unspecified.
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");

    let environment_filename = format!("{}.yaml", &environment.as_str());
    let settings = config::Config::builder()
        .add_source(config::File::from(config_dir.join("base.yaml")))
        .add_source(config::File::from(config_dir.join(environment_filename)))
        // Add in settings from environment variables (with a prefix of APP and '__' as separator)
        // E.g. `APP_APPLICATION__PORT=5001 would set `Settings.application.port`
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;

    match settings.try_deserialize::<Settings>() {
        Ok(settings) => {
            check_settings(&environment, &settings);
            Ok(settings)
        }
        Err(err) => Err(err.into()),
    }
}

#[derive(PartialEq, Eq)]
pub enum Environment {
    Local,
    Test,
    Prod,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Test => "test",
            Environment::Prod => "prod",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "test" => Ok(Self::Test),
            "production" => Ok(Self::Prod),
            other => Err(format!(
                "{} is not a supported environment. Use either `local`, 'test' or `prod`.",
                other
            )),
        }
    }
}
