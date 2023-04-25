use std::cmp::{max, min};
use std::ops::Deref;
use std::sync::Arc;
use std::sync::atomic::AtomicI64;

use thiserror::Error;

#[derive(Debug, Error)]
enum TokenError {
    #[error("Token Generation Failed {0}")]
    GenerationFailure(String),
}

pub enum SequenceFormat {
    Raw,
    PaddedInt(i8, String),
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

struct Token(String);

impl Deref for Token {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

trait TokenGenerator {
    fn generate(&self, format: TokenFormat) -> Result<Token, TokenError>;
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct InMemoryTokenGenerator {
    sequence: Arc<AtomicI64>
}

impl InMemoryTokenGenerator {
    pub fn new() -> InMemoryTokenGenerator {
        InMemoryTokenGenerator {
            sequence: Arc::new(AtomicI64::new(1))
        }
    }
}

impl TokenGenerator for InMemoryTokenGenerator {
    fn generate(&self,format: TokenFormat) -> Result<Token, TokenError> {
        todo!()
    }
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

impl Policy {
    pub fn generate(&self, token: String, value: String) -> String {
        let idx_left = min(self.keep_left as i32, value.len() as i32);
        let left = &value[0..idx_left as usize];

        let mut idx_right = value.len() as i32 - self.keep_right as i32;
        idx_right = min(value.len() as i32, max(idx_right, idx_left));

        let right = if idx_right >= 0 {
            &value[idx_right as usize..]
        } else {
            ""
        };
        std::format!(
            "{}{}{}{}",
            &self.prefix.clone().unwrap_or("".into()),
            left,
            token,
            right
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_nominal_case() {
        let policy = Policy {
            code: "sales".to_string(),
            format: TokenFormat::Sequence(SequenceFormat::Raw),
            prefix: Some("TOK-".to_string()),
            keep_left: 2,
            keep_right: 3,
        };

        assert_eq!(
            policy.generate("_1_".to_string(), "CARMEN MCCALLUM".to_string()),
            "TOK-CA_1_LUM"
        );
    }

    #[test]
    fn generate_with_value_too_short_for_keep_left() {
        let policy = Policy {
            code: "sales".to_string(),
            format: TokenFormat::Sequence(SequenceFormat::Raw),
            prefix: Some("TOK-".to_string()),
            keep_left: 4,
            keep_right: 0,
        };

        assert_eq!(
            policy.generate("_1_".to_string(), "ZO".to_string()),
            "TOK-ZO_1_"
        );
    }

    #[test]
    fn generate_with_value_too_short_for_keep_right() {
        let policy = Policy {
            code: "sales".to_string(),
            format: TokenFormat::Sequence(SequenceFormat::Raw),
            prefix: Some("TOK-".to_string()),
            keep_left: 0,
            keep_right: 4,
        };

        assert_eq!(
            policy.generate("_1_".to_string(), "ZO".to_string()),
            "TOK-_1_ZO"
        );
    }

    #[test]
    fn generate_with_value_when_keep_right_overlap_keep_left() {
        let policy = Policy {
            code: "sales".to_string(),
            format: TokenFormat::Sequence(SequenceFormat::Raw),
            prefix: Some("TOK-".to_string()),
            keep_left: 4,
            keep_right: 4,
        };

        assert_eq!(
            policy.generate("_1_".to_string(), "CARMEN".to_string()),
            "TOK-CARM_1_EN"
        );
    }
}
