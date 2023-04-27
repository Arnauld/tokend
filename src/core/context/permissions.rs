use crate::error::Error as CError;
use std::{collections::HashSet, fmt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Role {
    Root,
    Agent,
}

impl TryFrom<String> for Role {
    type Error = CError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "root" => Ok(Role::Root),
            "agent" => Ok(Role::Agent),
            _ => Err(CError::UnknownRole(value.to_string())),
        }
    }
}

impl Role {
    pub(crate) fn permissions(&self) -> HashSet<Permission> {
        let permissions = match self {
            Role::Root => vec![
                Permission::TenantCreate,
                Permission::TenantRead,
                Permission::TenantUpdate,
            ],
            Role::Agent => vec![
                Permission::AuditMetaRead,
                Permission::PolicyCreate,
                Permission::PolicyRead,
                Permission::PolicyUpdate,
                Permission::TokenCreate,
                Permission::TokenRead,
            ],
        };
        permissions.into_iter().collect()
    }
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub enum Permission {
    TenantCreate,
    TenantRead,
    TenantUpdate,
    //
    AuditMetaRead,
    //
    PolicyCreate,
    PolicyRead,
    PolicyUpdate,
    TokenCreate,
    TokenRead,
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Permission {
    pub fn is_tenant_required(&self) -> bool {
        match self {
            Permission::TenantCreate | Permission::TenantRead | Permission::TenantUpdate => false,
            _ => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Permission::*;

    #[test]
    fn role_from_string_case_insensitive() {
        for t in vec![
            ("AGENT".to_string(), Role::Agent),
            ("ROOT".to_string(), Role::Root),
        ] {
            let role: Role = t.0.try_into().unwrap();
            assert_eq!(role, t.1);
        }
    }

    #[test]
    fn role_from_string() {
        for t in vec![
            ("agent".to_string(), Role::Agent),
            ("root".to_string(), Role::Root),
        ] {
            let role: Role = t.0.try_into().unwrap();
            assert_eq!(role, t.1);
        }
    }

    #[test]
    fn role_from_unknown_string() {
        let res: Result<Role, CError> = "grumpf".to_string().try_into();
        assert!(res.is_err());
    }

    #[test]
    fn role_permissions_root() {
        let role = Role::Root;
        let perms = role.permissions();
        assert_eq!(&perms.contains(&TenantCreate), &true);
        assert_eq!(&perms.contains(&TenantRead), &true);
        assert_eq!(&perms.contains(&TenantUpdate), &true);
        //
        assert_eq!(&perms.contains(&AuditMetaRead), &false);
        assert_eq!(&perms.contains(&PolicyCreate), &false);
    }

    #[test]
    fn role_permissions_agent() {
        let role = Role::Agent;
        let perms = role.permissions();
        assert_eq!(&perms.contains(&TenantCreate), &false);
        assert_eq!(&perms.contains(&TenantRead), &false);
        assert_eq!(&perms.contains(&TenantUpdate), &false);
        //
        assert_eq!(&perms.contains(&AuditMetaRead), &true);
        assert_eq!(&perms.contains(&PolicyCreate), &true);
    }

    #[test]
    fn permission_is_tenant_required() {
        assert_eq!(TenantCreate.is_tenant_required(), false);
        assert_eq!(TenantRead.is_tenant_required(), false);
        assert_eq!(TenantUpdate.is_tenant_required(), false);
        //
        assert_eq!(AuditMetaRead.is_tenant_required(), true);
        assert_eq!(PolicyCreate.is_tenant_required(), true);
        assert_eq!(PolicyUpdate.is_tenant_required(), true);
        assert_eq!(PolicyRead.is_tenant_required(), true);
    }
}
