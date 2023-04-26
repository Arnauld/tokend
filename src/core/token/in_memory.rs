use std::ops::Deref;

use crate::core::token::{RawTokenGenerator, TokenError, TokenFormat};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

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
    fn generate(&self, token_format: &TokenFormat) -> Result<String, TokenError> {
        let raw_token = match &token_format {
            TokenFormat::Uuid => uuid::Uuid::new_v4().to_string(),
            TokenFormat::Sequence(formatter) => {
                let seq = self.sequence.deref().fetch_add(1, Ordering::SeqCst);
                formatter.apply(seq)
            }
        };
        Ok(raw_token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::token::SequenceFormat;

    #[test]
    fn in_memory_raw_token_generator_samples() {
        let generator = InMemoryRawTokenGenerator::new();
        let token_format = TokenFormat::Sequence(SequenceFormat::Raw);

        let seq1 = &generator.generate(&token_format);
        let seq2 = &generator.generate(&token_format);

        let x1 = seq1.as_ref().unwrap().deref();
        let x2 = seq2.as_ref().unwrap().deref();
        assert_ne!(x1, x2);

        assert_eq!(x1.parse::<i64>().unwrap(), 1);
        assert_eq!(x2.parse::<i64>().unwrap(), 2);
    }

    #[test]
    fn in_memory_raw_token_generator_uuid() {
        let generator = InMemoryRawTokenGenerator::new();
        let token_format = TokenFormat::Uuid;

        let seq1 = &generator.generate(&token_format);
        let seq2 = &generator.generate(&token_format);

        let x1 = seq1.as_ref().unwrap().deref();
        let x2 = seq2.as_ref().unwrap().deref();
        assert_ne!(x1, x2);

        let regex_pattern = r"^[a-z0-9]{8}-[a-z0-9]{4}-[a-z0-9]{4}-[a-z0-9]{4}-[a-z0-9]{12}$";
        let regex = regex::Regex::new(regex_pattern).unwrap();

        assert!(regex.is_match(x1));
        assert!(regex.is_match(x2));
    }
}
