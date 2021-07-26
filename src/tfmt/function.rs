pub enum StringFunction {
    /// Prepend with `String` to length `u64`
    Prepend(u64, String),
    /// Prepend with `"0"` to length `u64`
    LeadingZero(u64),
    /// Replace `String` with `String
    Replace(String, String),
    /// Split according to `sep`, return segment at `index`
    Split {
        sep: String,
        index: usize,
        max_split: u64,
    },
    ///Replaces invalid characters in `string`.
    ///
    /// Of limited usefulness, as Interpreter validates tags by default, but
    /// provided for completeness.
    Validate,
    /// Parses year from a calendar date.
    YearFromDate(String),
}

pub enum BooleanFunction {
    /// If `func.0` return `func.0` + `func.1`
    AndIf(String, String),
    /// If `func.0` return `func.1` else `func.2`
    If(String, String, String),
}

pub fn handle_function(string: String) -> String {
    "".to_string()
}
