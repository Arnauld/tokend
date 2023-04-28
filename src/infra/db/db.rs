use std::ops::{Deref, DerefMut};
use sqlx::{Pool, Postgres};
use sqlx::pool::PoolConnection;
use crate::core::context::ExecutionContext;
use crate::error::Error as TError;

pub struct ContextualizedPool {
    pool: Pool<Postgres>,
}

pub struct ContextualizedConnection(PoolConnection<Postgres>);

impl ContextualizedPool {
    pub fn new(pool: Pool<Postgres>) -> ContextualizedPool {
        ContextualizedPool { pool }
    }

    pub async fn acquire(
        &self,
        context: &ExecutionContext,
    ) -> Result<ContextualizedConnection, TError> {
        let conn = self.pool.acquire().await?;
        let mut contextualized = ContextualizedConnection(conn);
        contextualized.contextualize(context).await?;
        Ok(contextualized)
    }
}

impl Deref for ContextualizedConnection {
    type Target = PoolConnection<Postgres>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ContextualizedConnection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ContextualizedConnection {
    async fn contextualize(&mut self, context: &ExecutionContext) -> Result<(), TError> {
        set_config(
            self.deref_mut(),
            "var.caller_type".to_string(),
            context.caller.caller_type.to_string(),
        )
            .await?;

        set_config(
            self.deref_mut(),
            "var.caller_id".to_string(),
            context.caller.caller_id.to_string(),
        )
            .await?;

        let tid = match context.tenant.as_ref() {
            None => "".to_string(),
            Some(s) => s.to_string()
        };

        set_config(
            self.deref_mut(),
            "var.tenant_id".to_string(),
            tid,
        );

        Ok(())
    }
}

async fn set_config(
    conn: &mut PoolConnection<Postgres>,
    key: String,
    value: String,
) -> Result<(), TError> {
    sqlx::query!("select set_config($1, $2, 'f')", key, value)
        .fetch_one(conn)
        .await?;
    Ok(())
}