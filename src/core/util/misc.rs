pub fn left_pad<T>(value: T, length: usize, pad_char: char) -> String
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn left_pad_nominal_case() {
        assert_eq!(left_pad("HOG", 8, '_'), "_____HOG".to_string());
    }

    #[test]
    fn left_pad_nominal_empty_case() {
        assert_eq!(left_pad("", 8, '_'), "________".to_string());
    }

    #[test]
    fn left_pad_value_already_too_long() {
        assert_eq!(left_pad("HOGWARD", 4, '_'), "HOGWARD".to_string());
    }
}
