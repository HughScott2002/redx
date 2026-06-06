mod token;

pub use token::Token;

pub fn tokenize(input: &str) -> Vec<Token> {
    input
        .split_whitespace()
        .map(|part| {
            if part == "*" {
                Token::Pattern(part.to_string())
            } else {
                Token::Keyword(part.to_string())
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_keys_pattern() {
        assert_eq!(
            tokenize("KEYS *"),
            vec![Token::Keyword("KEYS".into()), Token::Pattern("*".into())]
        );
    }
}
