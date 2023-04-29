use crate::core::context::ExecutionContext;
use crate::core::util;
use async_trait::async_trait;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use crate::core::tenant::domain::{NewTenant, Tenant, Tenants};
use crate::error::{Error as CError, ErrorCode};
use crate::infra::{db, db::ContextualizedPool};

#[async_trait]
impl Tenants for ContextualizedPool {
    async fn declare_tenant(
        &self,
        context: &ExecutionContext,
        tenant: NewTenant,
    ) -> Result<Tenant, CError> {
        let mut conn = self.acquire(&context).await?;

        let res = sqlx::query!(
            "insert into tenants (code) values ($1) returning id,code",
            tenant.code.clone()
        )
        .fetch_one(conn.deref_mut())
        .await
        .map(|record| Tenant {
            id: record.id,
            code: record.code,
        })
        .map_err(|e| {
            if db::is_unique_constraint_error(&e, Some("tenants_code_key")) {
                CError::Generic(
                    ErrorCode::UniqueViolation,
                    "Duplicate tenant".to_string(),
                    HashMap::from([("code".to_string(), tenant.code)]),
                )
            } else {
                e.into()
            }
        })?;
        Ok(res)
    }

    async fn find_tenant_by_code(
        &self,
        context: &ExecutionContext,
        code: String,
    ) -> Result<Option<Tenant>, CError> {
        let mut conn = self.acquire(&context).await?;
        let res = sqlx::query!("select id, code from tenants where code = $1", code)
            .fetch_optional(conn.deref_mut())
            .await
            .map(|option_record| {
                option_record.map(|record| Tenant {
                    id: record.id,
                    code: record.code,
                })
            })?;
        Ok(res)
    }

    async fn find_tenants(
        &self,
        context: &ExecutionContext,
        paging: util::Paging,
    ) -> Result<util::Page<Tenant>, CError> {
        let mut conn = self.acquire(&context).await?;
        let limit: i64 = paging.first + 1;
        let cursor: util::paging::IntCursor = paging.clone().into();
        let res = sqlx::query!(
            "select id, code from tenants where id > $1 order by id limit $2",
            cursor.deref(),
            limit
        )
        .fetch_all(conn.deref_mut())
        .await
        .map(|records| {
            let tenants: Vec<Tenant> = records
                .into_iter()
                .map(|record| Tenant {
                    id: record.id,
                    code: record.code,
                })
                .collect();

            if tenants.len() > paging.first as usize {
                util::Page {
                    items: tenants[..tenants.len()].to_vec(),
                    page_infos: util::PageInfos::page_after(tenants.last().unwrap().id, true),
                }
            } else {
                util::Page {
                    items: tenants,
                    page_infos: util::PageInfos::no_page_after(),
                }
            }
        })?;
        Ok(res)
    }
}
