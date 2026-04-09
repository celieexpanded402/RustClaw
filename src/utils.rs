pub fn estimate_context_window_tokens(text: &str, max_token_length: usize) -> String {
    let tokens = text.split_whitespace().collect::<Vec<&str>>();
    if tokens.len() > max_token_length {
        tokens[..max_token_length].join(" ") + " ..."
    } else {
        text.to_string()
    }
}