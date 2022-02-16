use crate::error::{ErrorContext, FunctionError};
use crate::token::{Token, DIRECTORY_SEPARATORS, FORBIDDEN_GRAPHEMES};
use once_cell::sync::Lazy;
use regex::Regex;
use std::convert::TryFrom;

type Result<T> = std::result::Result<T, FunctionError>;

/// Wrapper function that delegates to TFMT functions.
pub(crate) fn handle_function<S, T>(
    input_text: &S,
    start_token: &Token,
    arguments: &[T],
) -> Result<String>
where
    S: AsRef<str>,
    T: AsRef<str>,
{
    validate(input_text, start_token, arguments)?;

    let name = start_token.get_string_unchecked();
    let arguments: Vec<&str> =
        arguments.iter().map(std::convert::AsRef::as_ref).collect();

    let function_output = match name {
        "prepend" => function_prepend(
            arguments[0],
            arguments[1].parse()?,
            arguments[2].parse()?,
        ),
        "num" => function_leading_zeroes(arguments[0], arguments[1].parse()?),
        "replace" => function_replace(arguments[0], arguments[1], arguments[2]),
        "split" => function_split(
            arguments[0],
            arguments[1],
            arguments[2].parse()?,
            arguments[3].parse()?,
        ),
        "validate" => function_validate(arguments[0]),
        "year_from_date" => function_year_from_date(arguments[0]),
        "andif" => function_andif(arguments[0], arguments[1]),
        "if" => function_if(arguments[0], arguments[1], arguments[2]),

        _ => panic!("Handled by validate!"),
    };

    Ok(function_output)
}

fn validate<S, T>(
    input_text: &S,
    start_token: &Token,
    arguments: &[T],
) -> Result<()>
where
    S: AsRef<str>,
    T: AsRef<str>,
{
    let name = start_token.get_string_unchecked();

    let required = match name {
        "validate" | "year_from_date" => 1,
        "andif" | "num" => 2,
        "if" | "prepend" | "replace" => 3,
        "split" => 4,
        _ => {
            return Err(FunctionError::UnknownFunction(
                ErrorContext::from_token(input_text, start_token),
                name.to_string(),
            ))
        }
    };

    let amount = arguments.len();

    if required == amount {
        Ok(())
    } else {
        Err(FunctionError::WrongArguments {
            context: ErrorContext::from_token(input_text, start_token),
            name: name.to_string(),
            expected: required,
            found: amount,
        })
    }
}

fn function_prepend(string: &str, length: usize, prefix: char) -> String {
    let prefix: String = std::iter::repeat(prefix)
        .take(length - (std::cmp::min(length, string.len())))
        .collect();

    prefix + string
}

fn function_leading_zeroes(string: &str, length: usize) -> String {
    function_prepend(string, length, '0')
}

fn function_replace(string: &str, from: &str, to: &str) -> String {
    string.replace(from, to)
}

fn function_split(
    string: &str,
    sep: &str,
    index: usize,
    max_split: isize,
) -> String {
    let vec: Vec<&str> = match usize::try_from(max_split) {
        Ok(max_split) => string.splitn(max_split, &sep).collect(),
        Err(_) => string.split(&sep).collect(),
    };

    vec.get(index)
        .map_or_else(|| "".to_string(), |s| (*s).to_string())
}

fn function_validate(string: &str) -> String {
    let mut out = String::from(string);

    FORBIDDEN_GRAPHEMES
        .iter()
        .for_each(|g| out = out.replace(g, ""));
    DIRECTORY_SEPARATORS
        .iter()
        .for_each(|g| out = out.replace(g, ""));

    out
}

fn function_year_from_date(string: &str) -> String {
    static REGEXES: Lazy<[Regex; 3]> = Lazy::new(|| {
        [
            Regex::new(r"^(?P<year>\d{4})$").unwrap(),
            Regex::new(r"^(?P<year>\d{4})-\d{2}-\d{2}$").unwrap(),
            Regex::new(r"^\d{2}-\d{2}-(?P<year>\d{4})$").unwrap(),
        ]
    });

    // re.captures returns None if there are not matches. We always have one
    // group, so if re.captures returns Some(), c[1] should never panic.
    REGEXES
        .iter()
        .map(|re| re.captures(string))
        .find(std::option::Option::is_some)
        .flatten()
        .map_or_else(|| "".to_string(), |c| c[1].to_string())
}

fn function_andif(condition: &str, true_string: &str) -> String {
    if condition.is_empty() {
        "".to_string()
    } else {
        format!("{}{}", condition, true_string)
    }
}

fn function_if(
    condition: &str,
    true_string: &str,
    false_string: &str,
) -> String {
    if condition.is_empty() {
        false_string.to_string()
    } else {
        true_string.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::TokenType;
    use anyhow::{bail, Result};

    #[test]
    fn function_test_wrong_arguments() -> Result<()> {
        let token = Token::new(TokenType::ID("prepend".to_string()), 0, 0);
        match handle_function(&"", &token, &["a", "b"]) {
            Ok(_) => bail!("prepend with 2 arguments did not raise an error!"),
            Err(FunctionError::WrongArguments { .. }) => (),
            Err(err) => bail!(
                "prepend with 2 arguments raised an unexpected error: {}!",
                err
            ),
        }

        let token =
            Token::new(TokenType::ID("year_from_date".to_string()), 0, 0);
        match handle_function(&"", &token, &["a", "b"]) {
            Ok(_) => bail!("year_from_date with 2 arguments did not raise an error!"),
            Err(FunctionError::WrongArguments{..}) => (),
            Err(err) => bail!("year_from_date with 2 arguments raised an unexpected error: {}!",err)
        }

        let token = Token::new(TokenType::ID("fake".to_string()), 0, 0);
        match handle_function(&"", &token, &["a"]) {
            Ok(_) => bail!("Unknown function did not raise an error!"),
            Err(FunctionError::UnknownFunction(..)) => (),
            Err(err) => {
                bail!("Unknown function raised an unexpected error: {}!", err)
            }
        }
        Ok(())
    }
}
