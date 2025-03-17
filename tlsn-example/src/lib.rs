use std::ffi::CString;
use std::os::raw::c_char;

mod tlsn_util;
use tlsn_util::run_tlsn_interactive;

#[unsafe(no_mangle)]
pub extern "C" fn say_hello() -> *const c_char {
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
    let c_string = CString::new(output).unwrap();
    c_string.into_raw()
}


