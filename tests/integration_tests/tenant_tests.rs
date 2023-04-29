use crate::helpers::startup;
use sqlx::postgres::PgPoolOptions;
use std::collections::HashSet;
use tokend::core::context::Permission::{TenantCreate, TenantRead};
use tokend::core::context::{Caller, CallerType, ExecutionContext};
use tokend::core::tenant::{NewTenant, Tenants};
use tokend::core::util::Paging;
use tokend::infra::config::{DatabaseRole, Settings};
use tokend::infra::db::ContextualizedPool;

fn sample_caller() -> Caller {
    Caller::new("007".to_string(), CallerType::USER)
}

#[tokio::test]
async fn create_new_tenants() {
    let (settings, repo) = set_up().await;
    let context = ExecutionContext::new(
        None,
        sample_caller(),
        HashSet::from([TenantRead, TenantCreate]),
    );

    // WHEN
    repo.declare_tenant(&context, NewTenant::new("idfm".to_string()))
        .await
        .expect("Failed to create tenant");

    // AND
    let result = repo
        .find_tenant_by_code(&context, "idfm".to_string())
        .await
        .expect("Failed to query tenant");
    assert!(result.is_some());
    assert_eq!(result.unwrap().code.as_str(), "idfm");

    let page = repo
        .find_tenants(&context, Paging::new(5, None))
        .await
        .expect("Failed to query tenants");
    assert_eq!(page.page_infos.has_next_page(), false);
    assert!(page.page_infos.after.is_none());
    assert_eq!(page.items.len(), 1);

    tear_down(&settings, repo).await;
}

async fn tear_down(settings: &Settings, repo: ContextualizedPool) {
    repo.close().await;
    startup::drop_db(&settings.database).await;
}

async fn set_up() -> (Settings, ContextualizedPool) {
    std::env::set_var("APP_ENVIRONMENT", "local");
    std::env::set_var("APP_CONFIG_DIR", "./conf");
    std::env::set_var("TEST_LOG", "true");
    let settings = startup::random_configuration().await;
    startup::spawn_db(&settings.database).await;
    startup::migrate_db(&settings.database).await;

    let connect_options = settings.database.with_db(&DatabaseRole::Application);
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect_with(connect_options)
        .await
        .expect("Failed to create connection pool");

    let repo = ContextualizedPool::new(pool);
    (settings, repo)
}
