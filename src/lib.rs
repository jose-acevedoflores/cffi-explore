use std::ffi::{c_void, CStr, CString};
use std::os::raw::{c_char, c_uchar};

struct RustSideHandler;

impl RustSideHandler {
    fn on_send(&self, src: &str, arg: &[u8], arg_len: usize) {
        println!("rust side on_send '{}' ", src, );
    }
}

#[repr(C)]
struct FFIWrapper {
    callback: extern "C" fn(*const RustSideHandler, *const c_char, *const c_uchar, usize),
    self_c_side: *const c_void,
    self_rust_side: *const RustSideHandler,
}

#[link(name = "dummy")]
extern "C" {

    // void send(const std::string& dest, const char* arg, size_t argLen);
    // void handler(const std::string& dest, Wrapper* p);
    fn send(dest: *const c_char, arg: *const c_uchar, arg_len: usize);
    fn handler(dest: *const c_char, ffi_obj: *mut FFIWrapper);
}

extern "C" fn handler_cb(
    rust_obj: *const RustSideHandler,
    dest: *const c_char,
    arg: *const c_uchar,
    arg_len: usize,
) {

    unsafe {
        let dest = CStr::from_ptr(dest);
        let sl = std::slice::from_raw_parts(arg, arg_len);
        (*rust_obj).on_send(dest.to_str().unwrap(), sl, arg_len);
    }
}

pub fn send_(dest: &str, data: &[u8]) {
    let dest = CString::new(dest).unwrap();
    unsafe {
        send(dest.as_ptr(), data.as_ptr(), data.len());
    }
}

pub fn handler_(dest: &str) {
    let rust_side_obj = Box::new(RustSideHandler {});

    let ffi_obj = Box::new(FFIWrapper {
        callback: handler_cb,
        self_c_side: std::ptr::null(),
        self_rust_side: std::boxed::Box::into_raw(rust_side_obj),
    });

    let ffi_obj = std::boxed::Box::into_raw(ffi_obj);
    let dest = CString::new(dest).unwrap();
    unsafe { handler(dest.as_ptr(), ffi_obj) }
}
