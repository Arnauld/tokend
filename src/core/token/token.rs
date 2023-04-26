use std::ops::Deref;
use secrecy::Secret;
use thiserror::Error;
use crate::core::util;

#[derive(Debug, Error)]
pub enum TokenError {
    #[error("Token Generation Failed {0}")]
    GenerationFailure(String),
}


#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token(String);

impl Deref for Token {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<String> for Token {
    fn from(value: String) -> Self {
        Token(value)
    }
}

pub enum SequenceFormat {
    Raw,
    PaddedInt(usize, char),
}

impl SequenceFormat {
    pub fn apply(&self, seq: i64) -> String {
        match self {
            SequenceFormat::Raw => seq.to_string(),
            SequenceFormat::PaddedInt(len, chr) => util::left_pad(seq, *len, *chr),
        }
    }
}


pub enum TokenFormat {
    /// UUID
    Uuid,

    /// Sequence based token; pattern is provided for formatting
    /// {1} corresponds to the current sequence
    /// {2} corresponds to the original value
    ///
    /// e.g.
    /// * (37, Raw) will produce "37"
    /// * (37, PaddedInt(4,"0")) will produce "0037"
    Sequence(SequenceFormat),
}


pub struct Policy {
    /// Unique identifier of the policy
    pub code: String,

    /// How the token are generated
    pub format: TokenFormat,

    /// Prefix of the token
    pub prefix: Option<String>,

    /// number of characters to retain from the left
    pub keep_left: usize,

    /// number of characters to retain from the right
    pub keep_right: usize,
}

pub trait TokenGenerator {
    fn generate(&self, policy: &Policy, value: Secret<String>) -> Result<Token, TokenError>;
}

pub trait RawTokenGenerator {
    fn generate(&self, token_format: &TokenFormat) -> Result<String, TokenError>;
}
