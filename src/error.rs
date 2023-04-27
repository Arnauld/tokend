//! Module for the error management
use std::collections::HashMap;
use std::fmt;
use thiserror::Error;

#[derive(Debug)]
pub enum ErrorCode {
    /// Generic error
    ServerError,
    /// when an entity cannot be created/updated
    UniqueViolation,
    /// when the directly targeted entity cannot be found
    NotFound,
    /// when a referenced entity cannot be found
    InvalidReference,
    /// when the user can fix the problem in the request itself
    BadRequest,
    /// Not Authorized
    Unauthorized,
    /// when the action would break a foreign key
    ReferenceViolation,
    /// when the actual data state does not permit the attempted action
    Forbidden,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// An error that can occur when processing GTFS data.
#[derive(Error, Debug)]
pub enum Error {
    /// Generic error
    #[error("Error {0}: {1}")]
    Generic(ErrorCode, String, HashMap<String, String>),

    #[error("Could not execute query {0}")]
    RepositoryError(String),

    /// A file references an Id that is not present
    #[error("The id {0} is not known")]
    ReferenceError(String),

    /// A config variable or setting is not present
    #[error("The config {0} is not defined")]
    MissingConfig(String),

    /// JWT token is not valid
    #[error("JWT token is invalid: {0}")]
    InvalidJWT(String),

    /// Tenant Id is not valid
    #[error("TenantId is invalid: {0}")]
    InvalidTenantId(String),

    /// Role is not known
    #[error("Role is not known: {0}")]
    UnknownRole(String),

    /// CallerType is not known
    #[error("Unknown CallerType: {0}")]
    UnknownCallerType(String),

    /// Propagate a ConfigError
    #[error("Unknown CallerType: {0}")]
    ConfigError(config::ConfigError)
}

impl From<config::ConfigError> for Error {
    fn from(err: config::ConfigError) -> Self {
        Self::ConfigError(err)
    }
}