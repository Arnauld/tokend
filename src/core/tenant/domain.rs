use crate::core::context::ExecutionContext;
use crate::core::util;
use async_trait::async_trait;

use crate::error::Error;

pub struct NewTenant {
    /// Unique identifier
    pub code: String,
}

impl NewTenant {
    pub fn new(code: String) -> NewTenant {
        NewTenant { code }
    }
}

#[derive(Debug, Clone)]
pub struct Tenant {
    /// Unique technical identifier
    pub id: i64,
    /// Unique identifier
    pub code: String,
}

#[async_trait]
pub trait Tenants {
    async fn declare_tenant(
        &self,
        context: &ExecutionContext,
        tenant: NewTenant,
    ) -> Result<Tenant, Error>;
    async fn find_tenant_by_code(
        &self,
        context: &ExecutionContext,
        code: String,
    ) -> Result<Option<Tenant>, Error>;
    async fn find_tenants(
        &self,
        context: &ExecutionContext,
        paging: util::Paging,
    ) -> Result<util::Page<Tenant>, Error>;
}
