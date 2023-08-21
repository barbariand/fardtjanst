#[macro_export]
macro_rules! remove_last_n_chars {
    ($s:expr, $n:expr) => {{
        let s = $s;
        let len = s.chars().count();
        let truncated_len = len.saturating_sub($n);
        &s[..s.char_indices().nth(truncated_len).map_or(s.len(), |(idx, _)| idx)]
    }};
}
