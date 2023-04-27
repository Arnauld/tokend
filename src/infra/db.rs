

pub struct ContextualizedPool {
    pool: Pool<Postgres>,
}

pub struct ContextualizedConnection(PoolConnection<Postgres>);

impl ContextualizedPool {
    pub fn new(pool: Pool<Postgres>) -> PgRepository {
        PgRepository { pool }
    }

    pub async fn acquire(
        &self,
        context: &ExecutionContext,
    ) -> Result<ContextualizedConnection, CError> {
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
    async fn contextualize(&mut self, context: &ExecutionContext) -> Result<(), CError> {
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

        Ok(())
    }
}

async fn set_config(
    conn: &mut PoolConnection<Postgres>,
    key: String,
    value: String,
) -> Result<(), CError> {
    sqlx::query!("select set_config($1, $2, 'f')", key, value)
        .fetch_one(conn)
        .await?;
    Ok(())
}