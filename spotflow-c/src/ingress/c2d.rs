use std::panic::AssertUnwindSafe;

use libc::{c_char, c_void, size_t};
use spotflow::{CloudToDeviceMessage, DeviceClient};

use crate::{
    call_safe_with_result, drop_str_ptr, ensure_logging,
    error::{update_last_error, CResult},
    ptr_to_ref, string_to_ptr,
};

/// The callback to process an incoming Cloud-to-Device Message. The callback is called only if you have configured it
/// using @ref spotflow_client_register_c2d_callback. The callback may be called on a separate thread. the Device SDK
/// uses the callback to processes the messages in the order they were received and only one message at a time.
/// When the callback returns the message including its properties will be deleted and the callback will not be invoked with the same message again.
/// All data that needs to be saved for later processing must be copied, pointers to properties and the message content will be invalidated when the
/// callback finishes. If the callback does not return (e.g. the application crashes during processing) the message will be delivered again on
/// subsequent runs.
///
/// @param msg The Cloud-to-Device Message. See @ref spotflow_c2d_message_t for details.
/// @param context The optional context that was configured in @ref spotflow_client_register_c2d_callback.
#[allow(non_camel_case_types)]
pub type C2dCallback = extern "C" fn(msg: *const C2dMessage, context: *mut c_void);

/// A Cloud-to-Device Message. This object is managed by the Device SDK and its contents must not be modified.
/// It's referenced from @ref spotflow_c2d_callback_t and the lifetime of @ref spotflow_c2d_message_t is the same as the
/// lifetime of the callback. If you need to keep the message for longer, copy it to your own memory.
#[repr(C)]
pub struct C2dMessage {
    /// (Don't modify) The length of the Cloud-to-Device Message content.
    content_length: size_t,
    /// (Don't modify) The content of the Cloud-to-Device Message. It shouldn't be interpreted as a C-style string,
    /// because it can contain binary data, including null characters. The length of the content is stored in
    /// @ref content_length.
    content: *const u8,
    /// (Don't modify) The number of properties in the Cloud-to-Device Message.
    properties_count: size_t,
    /// (Don't modify) The array of properties of the Cloud-to-Device Message. The length of the array is stored in
    /// @ref properties_count.
    properties: *const C2dProperty,
}

/// A property in a Cloud-to-Device Message. This object is managed by the Device SDK and its contents must not be modified.
/// It's referenced from @ref spotflow_c2d_message_t and the lifetime of @ref spotflow_c2d_property_t is the same as the
/// lifetime of the message. If you need to keep the property value for longer, copy it to your own memory.
#[repr(C)]
pub struct C2dProperty {
    /// (Don't modify) The name of the property.
    name: *const c_char,
    /// (Don't modify) The value of the property.
    value: *const c_char,
}

// This struct is used to pass around a (possibly null) void pointer that the C code provided
// It will be passed to the callback so that it has access to some state
// The pointer may be sent across thread boundaries
// We do not interact with the memory it points to in any way, all safety is the user's concern
struct Context(*mut c_void);
// This is needed so that the pointer can be passed to other threads.
// The user accessing the data behind the pointer is responsible for safety guarantees stemming from usage of the pointer (and the data behind it) in multiple threads
unsafe impl Send for Context {}

/// Register a function that will be invoked for every Cloud-to-Device Message received by the device.
/// Some of the messages may have been received earlier and were persisted in the device storage.
/// The callback will be invoked for all such messages as well. The callback may be invoked on a separate thread.
/// Use `context` to pass any data to the callback. See @ref spotflow_c2d_callback_t for details.
///
/// @param client The @ref spotflow_client_t object to register the callback for.
/// @param callback The callback to invoke for every Cloud-to-Device Message.
/// @param context (Optional) The context to pass to the callback. The data referenced by the pointer must be valid
///                until @ref spotflow_client_destroy is called. Use `NULL` if you don't need to pass any data.
/// @return @ref SPOTFLOW_OK if the callback was registered successfully, @ref SPOTFLOW_ERROR otherwise.
#[no_mangle]
#[allow(deprecated)] // We'll use the current interface until it's stabilized
                     /*pub*/
unsafe extern "C" fn spotflow_client_register_c2d_callback(
    client: *mut DeviceClient,
    callback: C2dCallback,
    context: *mut c_void,
) -> CResult {
    let client = AssertUnwindSafe(client);
    let result = call_safe_with_result(|| {
        ensure_logging();

        ptr_to_ref(*client)
    });
    let client = match result {
        Ok(ingress) => ingress,
        Err(e) => return e,
    };
    let context = Context(context);

    let callback = move |msg: &CloudToDeviceMessage| {
        let mut properties = Vec::with_capacity(msg.properties.len());
        for (key, value) in msg.properties.iter() {
            let key = string_to_ptr(key.clone());
            let value = string_to_ptr(value.clone());
            let property = C2dProperty { name: key, value };
            properties.push(property);
        }
        let c_msg = C2dMessage {
            content_length: msg.content.len(),
            content: msg.content.as_ptr(),
            properties_count: msg.properties.len(),
            properties: properties.as_ptr(),
        };

        // Disjoint capture -- we need the whole object because it implements Send while the *mut c_void contained therein does not (https://doc.rust-lang.org/edition-guide/rust-2021/disjoint-capture-in-closures.html)
        // *mut c_void does not implement Send and without this we get errors about that since only the pointer is captured
        _ = &context;
        callback(&c_msg, context.0);

        for property in properties {
            drop_str_ptr(property.name as *mut c_char);
            drop_str_ptr(property.value as *mut c_char);
        }
    };

    match client.process_c2d(callback) {
        Ok(_) => CResult::SpotflowOk,
        Err(e) => {
            update_last_error(e);
            CResult::SpotflowError
        }
    }
}
