use crate::core::context::Permission;
use crate::error::Error as CError;
use regex::Regex;
use std::fmt::Formatter;
use std::ops::Deref;
use std::{collections::HashSet, fmt};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CallerType {
    USER,
    SERVICE,
}

impl fmt::Display for CallerType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CallerType::USER => write!(f, "USER"),
            CallerType::SERVICE => write!(f, "SERVICE"),
        }
    }
}

impl TryFrom<String> for CallerType {
    type Error = crate::error::Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "user" => Ok(CallerType::USER),
            "service" => Ok(CallerType::SERVICE),
            s => Err(crate::error::Error::UnknownCallerType(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Caller {
    pub caller_id: String,
    pub caller_type: CallerType,
}

impl Caller {
    pub(crate) fn new(caller_id: String, caller_type: CallerType) -> Caller {
        Caller {
            caller_id,
            caller_type,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TenantId(String);

impl fmt::Display for TenantId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for TenantId {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<String> for TenantId {
    type Error = CError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        // at least two chars code
        let re = Regex::new(r"^[a-zA-Z0-9_-]{2,}$").unwrap();
        if re.is_match(s.as_str()) {
            Ok(TenantId(s))
        } else {
            Err(CError::InvalidTenantId(s))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionContext {
    pub caller: Caller,
    pub tenant: Option<TenantId>,
    permissions: HashSet<Permission>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionControlState {
    Authorized,
    Missing,
    TenantRequired,
}

impl fmt::Display for PermissionControlState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ExecutionContext {
    pub(crate) fn new(
        tenant: Option<TenantId>,
        caller: Caller,
        permissions: HashSet<Permission>,
    ) -> ExecutionContext {
        ExecutionContext {
            tenant,
            caller,
            permissions,
        }
    }

    pub fn has_permission(&self, permission: &Permission) -> PermissionControlState {
        if permission.is_tenant_required() && self.tenant.is_none() {
            return PermissionControlState::TenantRequired;
        }
        if self.permissions.contains(permission) {
            return PermissionControlState::Authorized;
        }
        PermissionControlState::Missing
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Permission::*;
    use PermissionControlState::*;

    #[test]
    fn caller_type_from_string_case_insensitive() {
        for t in vec![
            ("USER".to_string(), CallerType::USER),
            ("SERVICE".to_string(), CallerType::SERVICE),
        ] {
            let caller_type: CallerType = t.0.try_into().unwrap();
            assert_eq!(caller_type, t.1);
        }
    }

    #[test]
    fn caller_type_from_string() {
        for t in vec![
            ("user".to_string(), CallerType::USER),
            ("service".to_string(), CallerType::SERVICE),
        ] {
            let caller_type: CallerType = t.0.try_into().unwrap();
            assert_eq!(caller_type, t.1);
        }
    }

    #[test]
    fn caller_type_from_unknown_string() {
        let res: Result<CallerType, CError> = "grumpf".to_string().try_into();
        assert!(res.is_err());
    }

    #[test]
    fn caller_type_to_string() {
        assert_eq!("USER".to_string(), CallerType::USER.to_string());
        assert_eq!("SERVICE".to_string(), CallerType::SERVICE.to_string());
    }

    #[test]
    fn tenant_id_from_string() {
        let tid: Result<TenantId, CError> = "idfm".to_string().try_into();
        assert!(tid.is_ok());
        assert_eq!(tid.unwrap().deref(), &"idfm".to_string());
    }

    #[test]
    fn tenant_id_from_string_reject_empty_or_one_letter_string() {
        let tid: Result<TenantId, CError> = "".to_string().try_into();
        assert!(tid.is_err());
        match tid.err().unwrap() {
            CError::InvalidTenantId(_) => {}
            _ => panic!("Invalid error"),
        }

        let tid: Result<TenantId, CError> = "i".to_string().try_into();
        assert!(tid.is_err());
        match tid.err().unwrap() {
            CError::InvalidTenantId(_) => {}
            _ => panic!("Invalid error"),
        }
    }

    #[test]
    fn tenant_id_from_string_reject_not_ascii_char() {
        let tid: Result<TenantId, CError> = "az:er".to_string().try_into();
        assert!(tid.is_err());
    }

    fn sample_caller() -> Caller {
        Caller::new("007".to_string(), CallerType::USER)
    }

    #[test]
    fn execution_context_has_permission_when_tenant_is_missing() {
        let ctx = ExecutionContext::new(
            None,
            sample_caller(),
            HashSet::from([TenantRead, PolicyCreate, TokenCreate]),
        );
        assert_eq!(ctx.has_permission(&TenantRead), Authorized);
        assert_eq!(ctx.has_permission(&PolicyCreate), TenantRequired);
        assert_eq!(ctx.has_permission(&PolicyUpdate), TenantRequired);
        assert_eq!(ctx.has_permission(&PolicyRead), TenantRequired);
        assert_eq!(ctx.has_permission(&TokenRead), TenantRequired);
        assert_eq!(ctx.has_permission(&TokenCreate), TenantRequired);
    }

    #[test]
    fn execution_context_has_permission_when_tenant_is_present() {
        let tenant: TenantId = "idfm".to_string().try_into().unwrap();
        let ctx = ExecutionContext::new(
            Some(tenant),
            sample_caller(),
            HashSet::from([PolicyCreate, TenantRead, TokenRead]),
        );
        assert_eq!(ctx.has_permission(&TenantRead), Authorized);
        assert_eq!(ctx.has_permission(&PolicyCreate), Authorized);
        assert_eq!(ctx.has_permission(&PolicyUpdate), Missing);
        assert_eq!(ctx.has_permission(&PolicyRead), Missing);
        assert_eq!(ctx.has_permission(&TokenRead), Authorized);
        assert_eq!(ctx.has_permission(&TokenCreate), Missing);
    }
}
