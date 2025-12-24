/// Sanitize a rust identifier, thought for enum and struct name
pub fn sanitize_identifier(raw: &str) -> String {
    // If empty, return a fallback
    if raw.trim().is_empty() {
        return "Empty".to_string();
    }

    let sanitized: String = raw
        .trim()
        .chars()
        .map(|ch| match ch {
            // Common Punctuation
            ',' => "Comma".into(),
            ':' => "Colon".into(),
            ';' => "Semi".into(),
            '.' | '_' => "_".into(),

            // Math & Logic
            '+' => "PLUS".into(),
            '-' => "_".into(),
            '*' => "STAR".into(),
            '/' => "SLASH".into(),
            '=' => "EQUALS".into(),
            '%' => "PERCENT".into(),
            '<' => "LT".into(),
            '>' => "GT".into(),

            // Wrappers
            '(' => "".into(),
            ')' => "".into(),
            '[' => "OpenBracket".into(),
            ']' => "CloseBracket".into(),
            '{' => "OpenBrace".into(),
            '}' => "CloseBrace".into(),

            // Special / Web
            '@' => "At".into(),
            '#' => "Hash".into(),
            '$' => "Dollar".into(),
            '&' => "And".into(),
            '|' => "Pipe".into(),
            '!' => "Bang".into(),
            '?' => "Question".into(),
            '~' => "Tilde".into(),
            ' ' => "_".into(),

            // Quotes
            '"' => "Quote".into(),
            '\'' => "Tick".into(),
            '`' => "Backtick".into(),
            '\\' => "Backslash".into(),

            // Numbers
            '0' => "Zero".into(),
            '1' => "One".into(),
            '2' => "Two".into(),
            '3' => "Three".into(),
            '4' => "Four".into(),
            '5' => "Five".into(),
            '6' => "Six".into(),
            '7' => "Seven".into(),
            '8' => "Eight".into(),
            '9' => "Nine".into(),

            // Default
            c if c.is_alphanumeric() => c.to_string(),
            _ => String::new(), // Skip unknown chars or replace with "_"
        })
        .collect();

    // Ensure it doesn't start with a number (our mapping logic prevents this,
    // but as a safeguard if the mapping changes):
    if sanitized.chars().next().is_some_and(|c| c.is_numeric()) {
        format!("N{}", sanitized)
    } else {
        sanitized
    }
}
