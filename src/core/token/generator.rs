use crate::core::token::{
    Policy, RawTokenGenerator, Token, TokenError, TokenFormat, TokenGenerator,
};
use secrecy::{ExposeSecret, Secret, Zeroize};
use std::cmp::{max, min};
use std::ops::Deref;

#[derive(Clone, Debug)]
struct DefaultTokenGenerator<G>
where
    G: RawTokenGenerator,
{
    delegate: G,
}

impl<G> TokenGenerator for DefaultTokenGenerator<G>
where
    G: RawTokenGenerator,
{
    fn generate(&self, policy: &Policy, value: Secret<String>) -> Result<Token, TokenError> {
        let raw_token = self.delegate.generate(&policy.format)?;
        Ok(format(&policy, raw_token, value).into())
    }
}

pub fn format<T>(policy: &Policy, raw_token: String, value: Secret<T>) -> String
where
    T: ToString + Zeroize,
{
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
    use super::*;
    use crate::core::token::SequenceFormat;

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
            format(
                &policy,
                "_1_".to_string(),
                Secret::new("CARMEN MCCALLUM".to_string())
            ),
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
            format(&policy, "_1_".to_string(), Secret::new("ZO".to_string())),
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
            format(&policy, "_1_".to_string(), Secret::new("ZO".to_string())),
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
            format(
                &policy,
                "_1_".to_string(),
                Secret::new("CARMEN".to_string())
            ),
            "TOK-CARM_1_EN"
        );
    }
}
