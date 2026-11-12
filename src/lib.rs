use std::ffi::{CString, c_char};

#[unsafe(no_mangle)]
pub extern "C" fn add(left: u32, right: u32) -> u32 {
    left + right
}

#[unsafe(no_mangle)]
pub extern "C" fn substract(left: u32, right: u32) -> u32 {
    left - right
}

#[unsafe(no_mangle)]
pub extern "C" fn restructure(mod_path: *const c_char) {
    unsafe {
        let extraction_path_cstring = CString::from_raw(mod_path as *mut _);
    }
}
