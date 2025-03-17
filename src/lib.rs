use std::ffi::{CString, CStr};
use std::os::raw::c_char;

// Import logiki TLSNotary (przeniesione z `tlsn_util`)
mod tlsn_util;
use tlsn_util::run_tlsn_interactive;

#[no_mangle]
pub extern "C" fn say_hello() -> *const c_char {
    // Tworzymy tokio runtime i uruchamiamy async
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    let result = rt.block_on(async { run_tlsn_interactive().await });

    let output = match result {
        Ok(()) => {
            format!(
                "TLSNotary verified!",
            )
        }
        Err(e) => format!("TLSNotary error: {:?}", e),
    };

    // Konwersja do C-stringa
    let c_string = CString::new(output).unwrap();
    c_string.into_raw()
}


