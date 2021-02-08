use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uchar};

pub trait OnSend {
    fn on_send(&mut self, src: &str, arg: &[u8]);
}

//Allow dead code since both ptrs are only used by the C side
#[allow(dead_code)]
pub struct UserSpaceWrapper {
    ffi_wrapper: *mut FFIWrapper,
    ctx: *const FFICtx,
}

impl Drop for UserSpaceWrapper {
    fn drop(&mut self) {
        println!("free ffi_wrapper, ctx freed by c side");
        //TODO proper free
    }
}

#[repr(C)]
struct RustSideHandler {
    opaque: *mut dyn OnSend,
}

#[repr(C)]
struct FFIWrapper {
    callback: extern "C" fn(*mut RustSideHandler, *const c_char, *const c_uchar, usize),
    self_rust_side: *mut RustSideHandler,
}

#[repr(C)]
pub struct FFICtx {
    _private: [u8; 0],
}

#[link(name = "dummy")]
extern "C" {

    fn send(dest: *const c_char, arg: *const c_uchar, arg_len: usize) -> c_int;
    //NOTE: FFIWrapper includes a struct that has a trait object BUT it is not meant to be
    //      accessed by the c side so it should be sage.
    #[allow(improper_ctypes)]
    fn handler(dest: *const c_char, ffi_obj: *mut FFIWrapper) -> *const FFICtx;
    fn cancel(dest: *const c_char, ctx: *const FFICtx) -> c_int;
    fn shutdown();
}

extern "C" fn handler_cb(
    rust_obj: *mut RustSideHandler,
    dest: *const c_char,
    arg: *const c_uchar,
    arg_len: usize,
) {
    unsafe {
        let dest = CStr::from_ptr(dest);
        let sl = std::slice::from_raw_parts(arg, arg_len);
        (*(*rust_obj).opaque).on_send(dest.to_str().unwrap(), sl);
    }
}

pub fn send_(dest: &str, data: &[u8]) -> bool {
    let dest = CString::new(dest).unwrap();
    let res = unsafe { send(dest.as_ptr(), data.as_ptr(), data.len()) };

    res >= 0
}

pub fn handler_(dest: &str, handle: Box<dyn OnSend + Sync>) -> UserSpaceWrapper {
    let handle = std::boxed::Box::into_raw(handle);
    let rust_side_obj = Box::new(RustSideHandler { opaque: handle });

    let ffi_obj = Box::new(FFIWrapper {
        callback: handler_cb,
        self_rust_side: std::boxed::Box::into_raw(rust_side_obj),
    });

    let ffi_obj = std::boxed::Box::into_raw(ffi_obj);
    let dest = CString::new(dest).unwrap();
    let ctx = unsafe { handler(dest.as_ptr(), ffi_obj) };

    UserSpaceWrapper {
        ffi_wrapper: ffi_obj,
        ctx,
    }
}

pub fn cancel_(dest: &str, user_wrapper: UserSpaceWrapper) -> bool {
    let dest = CString::new(dest).unwrap();
    let res = unsafe { cancel(dest.as_ptr(), user_wrapper.ctx) };
    unsafe {
        //Important!
        // To free all resources held by the FFIWrapper struct we need to:
        //   - Rebuild the Box<FFIWrapper>
        //   - Rebuild the Box<RustSideHandler> held inside the FFIWrapper
        //   - Rebuild the Box<dyn OnSend> held inside the RustSideHandler
        // all these boxes will be dropped here, freeing the resources.
        let ffi_obj_to_drop = std::boxed::Box::from_raw(user_wrapper.ffi_wrapper);
        let self_rust_side_to_drop = std::boxed::Box::from_raw(ffi_obj_to_drop.self_rust_side);
        std::boxed::Box::from_raw(self_rust_side_to_drop.opaque);
    }
    res >= 0
}

pub fn shutdown_() {
    unsafe { shutdown() }
}

#[cfg(test)]
mod tests {
    use crate::OnSend;
    use std::mem::{size_of, transmute};

    struct TestStruct;
    impl OnSend for TestStruct {
        fn on_send(&mut self, _src: &str, _arg: &[u8]) {
            unimplemented!()
        }
    }
    struct TestStruct2;
    impl OnSend for TestStruct2 {
        fn on_send(&mut self, _src: &str, _arg: &[u8]) {
            unimplemented!()
        }
    }

    #[test]
    fn fat_ptr() {
        // https://iandouglasscott.com/2018/05/28/exploring-rust-fat-pointers/
        // So, this is a fat pointer.
        dbg!(size_of::<*mut dyn OnSend>());

        let handle: Box<dyn OnSend> = Box::new(TestStruct {});
        let handle = std::boxed::Box::into_raw(handle);

        dbg!(unsafe { transmute::<_, (usize, usize)>(handle) });
        dbg!(handle);

        let handle2: Box<dyn OnSend> = Box::new(TestStruct2 {});
        let handle2 = std::boxed::Box::into_raw(handle2);

        dbg!(unsafe { transmute::<_, (usize, usize)>(handle2) });
        dbg!(handle2);
    }
}
