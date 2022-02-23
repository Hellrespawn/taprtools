#![warn(missing_docs)]
#![warn(clippy::pedantic)]
//! Provides a test runner with a setup and teardown that runs even in the case
//! of panic.

use anyhow::Result;

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
///
/// Adapted from the following:
///
/// Opines, E. (2018, April 9). Test setup and teardown in Rust without a
/// framework. Medium. Retrieved February 23, 2022, from
/// https://medium.com/@ericdreichert/test-setup-and-teardown-in-rust-without-a-framework-ba32d97aa5ab
pub fn test_runner<S, T, F, X>(
    setup_function: S,
    teardown_function: T,
    test_function: F,
) -> Result<()>
where
    F: FnOnce(&X) -> Result<()> + std::panic::UnwindSafe,
    S: FnOnce() -> Result<X>,
    T: FnOnce(X) -> Result<()>,
    X: std::panic::UnwindSafe + std::panic::RefUnwindSafe,
{
    let setup_out = setup_function()?;

    let result = std::panic::catch_unwind(|| test_function(&setup_out));

    teardown_function(setup_out)?;

    assert!(result.is_ok());
    result.unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_function_err(_: &()) -> Result<()> {
        Err(anyhow::anyhow!("Woohoo!").into())
    }

    fn test_function_ok(_: &()) -> Result<()> {
        Ok(())
    }

    #[test]
    fn test_harness_with_err() -> Result<()> {
        let func = test_function_err;
        let harness = test_runner(|| Ok(()), |_| Ok(()), func);
        let bare = func(&());

        match bare {
            Ok(()) => assert!(harness.is_ok()),
            Err(_) => {
                // FIXME Find a way to compare the two errors, to see if they are the same.
                assert!(harness.is_err());
            }
        };

        Ok(())
    }

    #[test]
    fn test_harness_with_ok() -> Result<()> {
        let func = test_function_ok;
        let harness = test_runner(|| Ok(()), |_| Ok(()), func);
        let bare = func(&());

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
