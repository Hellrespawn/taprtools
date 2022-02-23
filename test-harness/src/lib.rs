#![warn(missing_docs)]
#![warn(clippy::pedantic)]
//! Provides a test runner with a setup and teardown that runs even in the case
//! of panic.

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Empty function for when setup or teardown isn't necessary.
///
/// # Errors
///
/// This function does not error, but still returns a Result to match the
/// signatures of `S` and `T` in `test_runner`.
pub fn none() -> Result<()> {
    Ok(())
}

/// Runs a test with a setup and safe teardown.
///
/// # Errors
///
/// This returns any errors in the setup-, teardown- and test-function
///
/// # Panics
///
/// This code uses `std::panic::catch_unwind` to catch any panic during testing
/// so the teardown function can be called. An assert statement later panics
/// again so the original trace is preserved and displayed to the user.
pub fn test_runner<F, S, T, U>(
    test_function: F,
    setup_function: S,
    teardown_function: T,
) -> Result<()>
where
    F: FnOnce(U) -> Result<()> + std::panic::UnwindSafe,
    S: FnOnce() -> Result<U>,
    T: FnOnce() -> Result<()>,
    U: std::panic::UnwindSafe,
{
    let setup_out = setup_function()?;

    let result = std::panic::catch_unwind(|| test_function(setup_out));

    teardown_function()?;

    assert!(result.is_ok());

    // The above asserts checks that result is Ok.
    result.unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_function(_: ()) -> Result<()> {
        Err(anyhow::anyhow!("Woohoo!").into())
    }

    #[test]
    fn test_harness_with_result() -> Result<()> {
        let func = test_function;
        let harness = test_runner(func, none, none);
        let bare = func(());

        match bare {
            Ok(()) => assert!(harness.is_ok()),
            Err(_) => {
                // FIXME Find a way to compare the two errors, to see if they are the same.
                assert!(harness.is_err());
            }
        };

        Ok(())
    }
}
