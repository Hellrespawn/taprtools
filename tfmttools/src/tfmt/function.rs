use super::error::FunctionError;
use crate::tfmt::token;
use once_cell::sync::Lazy;
use regex::Regex;
use std::convert::TryFrom;

type Result<T> = std::result::Result<T, FunctionError>;

fn validate(name: &str, amount: usize) -> Result<()> {
    let required = match name {
        "prepend" => 3,
        "num" => 2,
        "replace" => 3,
        "split" => 4,
        "validate" => 1,
        "year_from_date" => 1,
        "andif" => 2,
        "if" => 3,
        _ => return Err(FunctionError::UnknownFunction(name.to_string())),
    };

    if required != amount {
        Err(FunctionError::WrongArguments {
            name: name.to_string(),
            expected: required,
            found: amount,
        })
    } else {
        Ok(())
    }
}

/// Wrapper function that delegates to TFMT functions.
pub fn handle_function<S: AsRef<str>>(
    name: &str,
    arguments: &[S],
) -> Result<String> {
    let arguments: Vec<&str> = arguments.iter().map(|a| a.as_ref()).collect();

    validate(name, arguments.len())?;

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

        _ => return Err(FunctionError::UnknownFunction(name.to_string())),
    };

    Ok(function_output)
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
        .map(|s| s.to_string())
        .unwrap_or_else(|| "".to_string())
}

fn function_validate(string: &str) -> String {
    let mut out = String::from(string);

    token::FORBIDDEN_GRAPHEMES
        .iter()
        .for_each(|g| out = out.replace(g, ""));
    token::DIRECTORY_SEPARATORS
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
        .find(|e| e.is_some())
        .flatten()
        .map(|c| c[1].to_string())
        .unwrap_or_else(|| "".to_string())
}

fn function_andif(condition: &str, true_string: &str) -> String {
    if !condition.is_empty() {
        format!("{}{}", condition, true_string)
    } else {
        "".to_string()
    }
}

fn function_if(
    condition: &str,
    true_string: &str,
    false_string: &str,
) -> String {
    if !condition.is_empty() {
        true_string.to_string()
    } else {
        false_string.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{bail, Result};

    #[test]
    fn function_test_wrong_arguments() -> Result<()> {
        match handle_function("prepend", &["a", "b"]) {
            Ok(_) => bail!("prepend with 2 arguments did not raise an error!"),
            Err(FunctionError::WrongArguments { .. }) => (),
            Err(err) => bail!(
                "prepend with 2 arguments raised an unexpected error: {}!",
                err
            ),
        }

        match handle_function("year_from_date", &["a", "b"]) {
            Ok(_) => bail!("year_from_date with 2 arguments did not raise an error!"),
            Err(FunctionError::WrongArguments{..}) => (),
            Err(err) => bail!("year_from_date with 2 arguments raised an unexpected error: {}!",err)
        }

        match handle_function("fake", &["a"]) {
            Ok(_) => bail!("Unknown function did not raise an error!"),
            Err(FunctionError::UnknownFunction(..)) => (),
            Err(err) => {
                bail!("Unknown function raised an unexpected error: {}!", err)
            }
        }
        Ok(())
    }
}
