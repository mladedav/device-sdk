use anyhow::Error;
use std::{any::Any, cell::RefCell, cmp::min, slice};

use libc::{c_char, c_int, size_t};

use crate::{ensure_logging, SPOTFLOW_ERROR_MAX_LENGTH};

/// A result of a function that can fail. If a function returns a @ref spotflow_result_t, the caller must
/// check that the returned value is @ref SPOTFLOW_OK before continuing further. If the function returns
/// a different value, the caller can call @ref spotflow_read_last_error_message to retrieve the error message.
/// Check the documentation of the particular function to see what results it can return.
#[repr(C)]
pub enum CResult {
    /// The function succeeded.
    SpotflowOk = 0,
    /// The function failed with an error.
    SpotflowError,
    /// A buffer provided to the function is too small.
    SpotflowInsufficientBuffer,
    /// The client cannot provide the response because it is not connected to the platform.
    SpotflowNotReady,
    // This can be extended to accomodate more specific errors
}

thread_local! {
    static LAST_ERROR: RefCell<Option<Error>> = const { RefCell::new(None) };
}

/// Update the most recent error, clearing whatever may have been there before.
pub(crate) fn update_last_error(err: Error) {
    log::info!("Setting LAST_ERROR: {}", err);

    {
        // Print a pseudo-backtrace for this error, following back each error's
        // cause until we reach the root error.
        let mut cause = err.source();
        while let Some(parent_err) = cause {
            log::debug!("Caused by: {parent_err}");
            // // Currently ureq does not allow us to do this because response is behind a reference and there is no way to read the content
            // if let Some(ureq::Error::Status(_, response)) = parent_err.downcast_ref::<ureq::Error>() {
            //     if let Ok(content) = response.into_string() {
            //         log::debug!("Response content:\n{content}");
            //     }
            // }
            cause = parent_err.source();
        }
    }

    LAST_ERROR.with(|prev| {
        *prev.borrow_mut() = Some(err);
    });
}

pub(crate) fn update_last_error_with_panic(panic: Box<dyn Any + Send>) {
    match panic.downcast::<&str>() {
        Ok(str) => update_last_error(anyhow::anyhow!(str)),
        Err(e) => match e.downcast::<String>() {
            Ok(str) => update_last_error(anyhow::anyhow!(str)),
            Err(_) => update_last_error(anyhow::anyhow!(
                "Unknown panic with no string representation."
            )),
        },
    }
}

/// Retrieve the most recent error, clearing it in the process.
pub(crate) fn take_last_error() -> Option<Error> {
    LAST_ERROR.with(|prev| prev.borrow_mut().take())
}

/// Write the most recent error message into the provided buffer as a UTF-8 string, returning the
/// number of bytes written. If successful, the error message is consumed and will not be returned
/// again in a future call to this function.
///
/// Since the string is in the UTF-8 encoding, Windows users may need to convert it to a UTF-16
/// string before displaying it.
///
/// @param buffer The buffer to write the error message into.
/// @param buffer_length The length of the buffer in bytes. Use @ref SPOTFLOW_ERROR_MAX_LENGTH to be sure that it is
///                      always large enough.
/// @return The number of bytes written into the buffer including the trailing null character.
///         If no error has been produced yet, returns 0. If `buffer` or `buffer_length` are invalid
///         (for example, null pointer or insufficient length), returns -1.
#[no_mangle]
pub unsafe extern "C" fn spotflow_read_last_error_message(
    buffer: *mut c_char,
    buffer_length: size_t,
) -> c_int {
    ensure_logging();

    if buffer.is_null() {
        log::warn!("Null pointer passed into last_error_message() as the buffer.");
        return -1;
    }

    let last_error = match take_last_error() {
        Some(err) => err,
        None => return 0,
    };

    let error_message = last_error.to_string();

    // Don't return the error if the user provided a buffer long at least SPOTFLOW_ERROR_MAX_LENGTH but the actual error message is longer
    // (that's basically a problem on the Device SDK side)
    if error_message.len() >= buffer_length && buffer_length < SPOTFLOW_ERROR_MAX_LENGTH {
        log::warn!("Buffer provided for writing the last error message is too small. Needed at least {} bytes but got {}.",
            error_message.len() + 1,
            buffer_length
        );
        // Put the error back in case the caller wants to try again
        update_last_error(last_error);
        return -1;
    }

    let buffer = slice::from_raw_parts_mut(buffer as *mut u8, buffer_length);

    // Gracefully handle the case when the actual error message is longer than SPOTFLOW_ERROR_MAX_LENGTH and the buffer
    // is at least SPOTFLOW_ERROR_MAX_LENGTH bytes long - truncate the error message to fit into the buffer
    let copy_length = min(error_message.len(), buffer_length - 1);

    std::ptr::copy_nonoverlapping(error_message.as_ptr(), buffer.as_mut_ptr(), copy_length);

    // Add a trailing null so people using the string as a `char *` don't
    // accidentally read into garbage.
    buffer[copy_length] = 0;

    (copy_length + 1) as c_int
}
