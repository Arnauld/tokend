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
