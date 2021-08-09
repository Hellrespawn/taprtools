use thiserror::Error;

// TODO Expand tfmt error handling.

#[derive(Error, Debug, PartialEq)]
/// Error from the [token] module.
pub enum TokenError {
    #[error(r#""{0}" is not a valid TokenType!"#)]
    /// Invalid [TokenType].
    InvalidType(String),
}

#[derive(Error, Debug, PartialEq)]
/// Error from the [lexer] module.
pub enum LexerError {
    /// [Lexer] exhausted text input stream.
    #[error(r#"Lexer exhausted text input stream looking for "{0}"!"#)]
    ExhaustedText(String),

    /// String contains forbidden grapheme.
    #[error(r#"String contains forbidden grapheme "{0}"!"#)]
    ForbiddenGrapheme(String),

    /// String contains forbidden grapheme.
    #[error(r#"String contains newline character: "{0}""#)]
    NewlineInString(String),

    /// String contains forbidden grapheme.
    #[error(r#"Input ends with "{found}", expected "{expected}""#)]
    WrongTerminatorAtEOF { found: String, expected: String },
}
