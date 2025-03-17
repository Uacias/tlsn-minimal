use tlsn_minimal::say_hello;

use std::ffi::CStr;
use std::os::raw::c_char;


fn main() {
    unsafe {
        let ptr = say_hello();
        if ptr.is_null() {
            println!("Received null pointer");
            return;
        }

        let message = CStr::from_ptr(ptr).to_string_lossy();
        println!("Received: {}", message);
    }
}
