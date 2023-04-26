use std::cmp::{max, min};
use std::ops::Deref;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use secrecy::{Secret, ExposeSecret, Zeroize};
use crate::core::token::{Policy, RawTokenGenerator, Token, TokenFormat, TokenGenerator, TokenError};

#[derive(Clone, Debug)]
struct InMemoryRawTokenGenerator {
    sequence: Arc<AtomicI64>,
}

impl InMemoryRawTokenGenerator {
    pub fn new() -> InMemoryRawTokenGenerator {
        InMemoryRawTokenGenerator {
            sequence: Arc::new(AtomicI64::new(1)),
        }
    }
}

impl RawTokenGenerator for InMemoryRawTokenGenerator {
    fn generate(&self, policy: &Policy) -> Result<String, TokenError> {
        let raw_token = match &policy.format {
            TokenFormat::Uuid => uuid::Uuid::new_v4().to_string(),
            TokenFormat::Sequence(formatter) => {
                let seq = self.sequence.deref().fetch_add(1, Ordering::SeqCst);
                formatter.apply(seq)
            }
        };
        Ok(raw_token)
    }
}

#[derive(Clone, Debug)]
struct DefaultTokenGenerator<G> where G: RawTokenGenerator {
    delegate: G,
}

impl<G> TokenGenerator for DefaultTokenGenerator<G>
    where G: RawTokenGenerator {
    fn generate(&self, policy: &Policy, value: Secret<String>) -> Result<Token, TokenError> {
        let raw_token = self.delegate.generate(&policy)?;
        Ok(format(&policy, raw_token, value).into())
    }
}


pub fn format<T>(policy: &Policy, raw_token: String, value: Secret<T>) -> String where T: ToString + Zeroize {
    let unsecure = value.expose_secret().to_string();
    let idx_left = min(policy.keep_left as i32, unsecure.len() as i32);
    let left = &unsecure[0..idx_left as usize];

    let mut idx_right = unsecure.len() as i32 - policy.keep_right as i32;
    idx_right = min(unsecure.len() as i32, max(idx_right, idx_left));

    let right = if idx_right >= 0 {
        &unsecure[idx_right as usize..]
    } else {
        ""
    };
    std::format!(
        "{}{}{}{}",
        &policy.prefix.clone().unwrap_or("".into()),
        left,
        raw_token,
        right
    )
}

#[cfg(test)]
mod tests {
    use crate::core::token::SequenceFormat;
    use super::*;

    #[test]
    fn format_nominal_case() {
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
    fn format_with_value_too_short_for_keep_left() {
        let policy = Policy {
            code: "sales".to_string(),
            format: TokenFormat::Sequence(SequenceFormat::Raw),
            prefix: Some("TOK-".to_string()),
            keep_left: 4,
            keep_right: 0,
        };

        assert_eq!(
            policy.format("_1_".to_string(), "ZO".to_string()),
            "TOK-ZO_1_"
        );
    }

    #[test]
    fn format_with_value_too_short_for_keep_right() {
        let policy = Policy {
            code: "sales".to_string(),
            format: TokenFormat::Sequence(SequenceFormat::Raw),
            prefix: Some("TOK-".to_string()),
            keep_left: 0,
            keep_right: 4,
        };

        assert_eq!(
            policy.format("_1_".to_string(), "ZO".to_string()),
            "TOK-_1_ZO"
        );
    }

    #[test]
    fn format_with_value_when_keep_right_overlap_keep_left() {
        let policy = Policy {
            code: "sales".to_string(),
            format: TokenFormat::Sequence(SequenceFormat::Raw),
            prefix: Some("TOK-".to_string()),
            keep_left: 4,
            keep_right: 4,
        };

        assert_eq!(
            policy.format("_1_".to_string(), "CARMEN".to_string()),
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

        assert!(regex.is_match(x1));
        assert!(regex.is_match(x2));
    }
}
