//! Dummy library wrapper!
//! Provide a safe abstraction over libdummy
//!
use crate::ext::{FFICtx, FFIWrapper, RustSideHandler};
use std::ffi::CString;
use std::ptr::null;

pub trait OnSend {
    fn on_send(&mut self, src: &str, arg: &[u8]);
}

/// Keep track of a given `handle: Box<dyn OnSend + Sync>` registered via the [`handler`] function.
//Allow dead code since both ptrs are only used by the C side
#[allow(dead_code)]
pub struct UserSpaceWrapper {
    ffi_wrapper: *mut FFIWrapper,
    ctx: *const FFICtx,
}

impl Drop for UserSpaceWrapper {
    fn drop(&mut self) {
        //NOTE: the UserSpaceWrapper object is normally associated with a 'dest'.
        // If the wrong string is passed to the 'libdummy' side it won't free the 'ctx' variable
        // and it would be a memory leak. In the real library that won't be a problem.

        //NOTE: if this is called AFTER the library was shutdown it will segfault.
        //      Potential solution would be: the shutdown method setting a flag read here
        //      (SyncLazy looks promising)
        let res = self.delete("");
        println!("dropped UserSpaceWrapper, ctx freed:'{}'", res);
    }
}

impl UserSpaceWrapper {
    fn delete(&mut self, dest: &str) -> bool {
        if self.ctx == null() {
            return false;
        }
        let dest = CString::new(dest).unwrap();

        //Safety: calling extern function. This is valid as long as shutdown hasn't been called
        let res = unsafe { crate::ext::cancel(dest.as_ptr(), self.ctx) };

        //Safety: The boxes are created in 'new' and immediately consumed to raw ptrs.
        //        They are only ever read again in here just to drop them so they will
        //        be valid
        unsafe {
            //Important!
            // To free all resources held by the FFIWrapper struct we need to:
            //   - Rebuild the Box<FFIWrapper>
            //   - Rebuild the Box<RustSideHandler> held inside the FFIWrapper
            //   - Rebuild the Box<dyn OnSend> held inside the RustSideHandler
            // all these boxes will be dropped here, freeing the resources.
            let ffi_obj_to_drop = std::boxed::Box::from_raw(self.ffi_wrapper);
            let self_rust_side_to_drop = std::boxed::Box::from_raw(ffi_obj_to_drop.self_rust_side);
            std::boxed::Box::from_raw(self_rust_side_to_drop.opaque);
        }
        self.ctx = null();
        res >= 0
    }

    fn new(dest: &str, handle: Box<dyn OnSend + Sync>) -> Self {
        let handle = std::boxed::Box::into_raw(handle);
        let rust_side_obj = Box::new(RustSideHandler { opaque: handle });

        let ffi_obj = Box::new(FFIWrapper {
            callback: crate::ext::handler_cb,
            self_rust_side: std::boxed::Box::into_raw(rust_side_obj),
        });

        let ffi_obj = std::boxed::Box::into_raw(ffi_obj);
        let dest = CString::new(dest).unwrap();

        //Safety: calling extern function. This is valid as long as shutdown hasn't been called
        let ctx = unsafe { crate::ext::handler(dest.as_ptr(), ffi_obj) };

        UserSpaceWrapper {
            ffi_wrapper: ffi_obj,
            ctx,
        }
    }
}

/// Private module that encapsulates all the extern parts of the library.
mod ext {
    use crate::OnSend;
    use std::ffi::CStr;
    use std::os::raw::{c_char, c_int, c_uchar};

    /// Struct introduced in order to send the fat ptr that represents an OnSend trait object
    /// through ffi.
    /// See [Passing dyn trait through ffi](../notes/fatptr_through_ffi.md)
    #[repr(C)]
    pub struct RustSideHandler {
        pub opaque: *mut dyn OnSend,
    }

    /// Structure defined by the libdummy header for data exchange.
    #[repr(C)]
    pub struct FFIWrapper {
        /// Function pointer used by the library to reach back
        pub callback: extern "C" fn(*mut RustSideHandler, *const c_char, *const c_uchar, usize),
        /// Entity that is meant to handle the callback. This field will be passed in as the
        /// first arg of the fn ptr above.
        pub self_rust_side: *mut RustSideHandler,
    }

    /// Represents the extern ptr to the Context struct given by the library.
    /// As long as this ptr is valid, the library will reach back to rust via the 'handler_cb' when
    /// callbacks occur. The FFICtx will be invalidated after a call to 'cancel'.
    /// This is an opaque struct not meant to be accessed by rust.
    #[repr(C)]
    pub struct FFICtx {
        _private: [u8; 0],
    }

    #[link(name = "dummy")]
    extern "C" {
        /// Sends a series of bytes to the given `dest`
        /// # Arguments
        /// * `dest` - null terminated string
        /// * `arg` - byte array of the data to be sent
        /// * `arg_len` - length of the `arg` byte array
        ///
        /// Returns an int where '>=0' is success
        ///
        pub fn send(dest: *const c_char, arg: *const c_uchar, arg_len: usize) -> c_int;

        /// Register a handler on the given `dest`
        /// # Arguments
        /// * `dest` - null terminated string
        /// * `ffi_obj` - handler data to be used by libdummy
        ///
        /// Returns a context struct that corresponds to the given `ffi_obj`
        //NOTE: FFIWrapper includes a struct that has a trait object BUT it is not meant to be
        //      accessed by the c side so it should be safe.
        #[allow(improper_ctypes)]
        pub fn handler(dest: *const c_char, ffi_obj: *mut FFIWrapper) -> *const FFICtx;

        /// Sends a series of bytes to the given `dest`
        /// # Arguments
        /// * `dest` - null terminated string
        /// * `ctx` - ctx struct to cancel.
        /// Returns an int where '>=0' is success
        pub fn cancel(dest: *const c_char, ctx: *const FFICtx) -> c_int;
        ///Completely shutdown libdummy. After this call, no other extern method is valid.
        pub fn shutdown();
    }

    /// Function callback used by the library to reach back to rust.
    pub extern "C" fn handler_cb(
        rust_obj: *mut RustSideHandler,
        dest: *const c_char,
        arg: *const c_uchar,
        arg_len: usize,
    ) {
        //Safety: This is the most critical unsafe block.
        // This block assumes the C library honors its contract and will NOT trigger this callback
        // with a RustSideHandler that has already been freed. As a reminder, a
        // RustSideHandler comes paired up with an FFICtx. Once the FFCtx is returned to the C via
        // 'cancel' the associated RustSideHandler is freed.
        unsafe {
            let dest = CStr::from_ptr(dest);
            let sl = std::slice::from_raw_parts(arg, arg_len);
            (*(*rust_obj).opaque).on_send(dest.to_str().unwrap(), sl);
        }
    }
}

/// Sends a series of bytes to the given `dest`
/// # Arguments
/// * `dest` - destination for the data
/// * `data` - byte array of the data to be sent
///
/// Returns true if operation is a success
///
pub fn send(dest: &str, data: &[u8]) -> bool {
    let dest = CString::new(dest).unwrap();

    //Safety: calling extern function. This is valid as long as shutdown hasn't been called
    let res = unsafe { crate::ext::send(dest.as_ptr(), data.as_ptr(), data.len()) };

    res >= 0
}

/// Register a handler on the given `dest`
/// # Arguments
/// * `dest` - route the given `handler` should receive data on.
/// * `handle` - handler data to be used by libdummy
///
/// Returns a context struct that corresponds to the given `ffi_obj`
pub fn handler(dest: &str, handle: Box<dyn OnSend + Sync>) -> UserSpaceWrapper {
    UserSpaceWrapper::new(dest, handle)
}

/// Cancel a `dest`/`user_wrapper` combination. This should correspond to the ones received by a call
/// to [`handler`]
/// # Arguments
/// * `dest` - same route used that produced the given `user_wrapper`
/// * `user_wrapper` - handler to cancel.
pub fn cancel(dest: &str, user_wrapper: UserSpaceWrapper) -> bool {
    let mut user_wrapper = user_wrapper;
    user_wrapper.delete(dest)
}

///Completely shutdown libdummy. After this call, no other extern method is valid.
pub fn shutdown() {
    //Safety: calling extern function.
    unsafe { crate::ext::shutdown() }
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
