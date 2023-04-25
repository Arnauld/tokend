use std::cmp::{max, min};
use std::ops::Deref;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

use thiserror::Error;

#[derive(Debug, Error)]
enum TokenError {
    #[error("Token Generation Failed {0}")]
    GenerationFailure(String),
}

pub enum SequenceFormat {
    Raw,
    PaddedInt(usize, char),
}

fn left_pad<T>(value: T, length: usize, pad_char: char) -> String
where
    T: ToString,
{
    let value_str = value.to_string();
    let value_len = value_str.len();
    if value_len >= length {
        return value_str;
    }
    let pad_len = length - value_len;
    let pad_str = pad_char.to_string().repeat(pad_len);
    format!("{}{}", pad_str, value_str)
}

impl SequenceFormat {
    pub fn apply(&self, seq: i64) -> String {
        match self {
            SequenceFormat::Raw => seq.to_string(),
            SequenceFormat::PaddedInt(len, chr) => left_pad(seq, *len, *chr),
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

#[derive(Clone, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug)]
struct InMemoryTokenGenerator {
    sequence: Arc<AtomicI64>,
}

impl InMemoryTokenGenerator {
    pub fn new() -> InMemoryTokenGenerator {
        InMemoryTokenGenerator {
            sequence: Arc::new(AtomicI64::new(1)),
        }
    }
}

impl TokenGenerator for InMemoryTokenGenerator {
    fn generate(&self, format: TokenFormat) -> Result<Token, TokenError> {
        match format {
            TokenFormat::Uuid => Ok(Token(uuid::Uuid::new_v4().to_string())),
            TokenFormat::Sequence(format) => {
                let seq = self.sequence.deref().fetch_add(1, Ordering::SeqCst);
                Ok(Token(format.apply(seq)))
            }
        }
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

    #[test]
    fn in_memory_token_generator_samples() {
        let generator = InMemoryTokenGenerator::new();

        let seq1 = &generator.generate(TokenFormat::Sequence(SequenceFormat::Raw));
        let seq2 = &generator.generate(TokenFormat::Sequence(SequenceFormat::Raw));

        let x1 = seq1.as_ref().unwrap().deref();
        let x2 = seq2.as_ref().unwrap().deref();
        assert_ne!(x1, x2);

        assert_eq!(x1.parse::<i64>().unwrap(), 1);
        assert_eq!(x2.parse::<i64>().unwrap(), 2);
    }

    #[test]
    fn in_memory_token_generator_uuid() {
        let generator = InMemoryTokenGenerator::new();

        let seq1 = &generator.generate(TokenFormat::Uuid);
        let seq2 = &generator.generate(TokenFormat::Uuid);

        let x1 = seq1.as_ref().unwrap().deref();
        let x2 = seq2.as_ref().unwrap().deref();
        assert_ne!(x1, x2);

        let regex_pattern = r"^[a-z0-9]{8}-[a-z0-9]{4}-[a-z0-9]{4}-[a-z0-9]{4}-[a-z0-9]{12}$";
        let regex = regex::Regex::new(regex_pattern).unwrap();

        print!(">>>{:?}<<<", x1);
        assert!(regex.is_match(x1));
        assert!(regex.is_match(x2));
    }
}
