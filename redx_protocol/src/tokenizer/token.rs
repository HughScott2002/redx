use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Keyword(String),
    Pattern(String),
}

impl Token {
    pub fn value(&self) -> &str {
        match self {
            Token::Keyword(value) | Token::Pattern(value) => value,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "TOKEN: {}", self.value())
    }
}
