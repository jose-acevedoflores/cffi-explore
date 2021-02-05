## The even better, ridiculously amazing `cffi-explore`


This project is a `rust` wrapper around a hypothetical `c++` library
(see [libdummy](libdummy/README.md) ) that exposes a `C` abi.
The goal of this repo is to document/explore how to write such a `rust` wrapper 
from the perspective of someone not only new to `rust`, but new to writing
non garbage collected code in general.



One very **important** caveat is that my `c/c++` knowledge is **very** limited.
I've lived most of my life in the `Java/JS/Python` land of GC goodness.
This is an attempt to get my feet wet on the mystical world where ___memory management___
is ... manual :scream:



## Notes to `&self`

I liked the idea of providing a Trait object (`OnSend`) to the
`handler_(dest: &str, handle: Box<dyn OnSend>) ...` function
instead of a concrete struct, so the users of the library could just provide an arbitrary struct
that satisfies the bound (or maybe I'm pigeonholed in java).

Where I ran into trouble was when I tried passing the resulting
`*mut dyn OnSend` down to the `c` side and back through the `extern "C" fn handler_cb`

The code looked like this
```rust
#[link(name = "dummy")]
extern "C" {
    fn handler(dest: *const c_char, ffi_obj: *mut FFIWrapper) -> *const FFICtx;
}

#[repr(C)]
struct FFIWrapper {
    callback: extern "C" fn(*mut dyn OnSend, *const c_char, *const c_uchar, usize),
    self_rust_side: *mut dyn OnSend,
}

extern "C" fn handler_cb(
    rust_obj: *mut dyn OnSend,
    dest: *const c_char,
    arg: *const c_uchar,
    arg_len: usize,
) {
    unsafe {
        let dest = CStr::from_ptr(dest);
        let sl = std::slice::from_raw_parts(arg, arg_len);
        println!("Fat ptr passed to callback: {:?}",
                 unsafe { transmute::<_, (usize, usize)>(rust_obj) });

        let mut bv = std::boxed::Box::from_raw(rust_obj);
        bv.as_mut().on_send(dest.to_str().unwrap(), sl); // <--- SEGFAULTS HERE
    }
}


pub fn handler_(dest: &str, handle: Box<dyn OnSend>) -> UserSpaceWrapper {
    let handle = std::boxed::Box::into_raw(handle);

    println!("Original fat ptr:{:?}", unsafe { transmute::<_, (usize, usize)>(handle) });

    let ffi_obj = Box::new(FFIWrapper {
        callback: handler_cb,
        self_rust_side: handle,
    });

    let ffi_obj = std::boxed::Box::into_raw(ffi_obj);
    let dest = CString::new(dest).unwrap();
    let ctx = unsafe { handler(dest.as_ptr(), ffi_obj) };

    UserSpaceWrapper {
        ffi_wrapper: ffi_obj,
        ctx,
    }
}
```

In essence, the `handler_` function packs everything inside the `FFIWrapper` and then
sends it via the exposed `handler` method. Eventually, the `c` code reaches back via `handler_cb`
where the `rust_obj` is the ptr to the user provided handler that impls `OnSend`.
The Box is then rebuilt, and I try to invoke the on_send aaannddd... :boom::boom::boom: segfault.
After banging my head for a bit I stumbled upon
[this article](https://iandouglasscott.com/2018/05/28/exploring-rust-fat-pointers/)
which gives an insight to rust fat pointers. If I understood correctly, `dyn Traits` are
DSTs so they are represented by fat pointers (16 bytes vs 8 bytes in a 64bit machine).
The first 8 bytes are the data ptr and the next 8 bytes point to the vtable.
With the code above, if you do a `transmute` on the `*mut dyn OnSend` before and after you get
```shell
                 (    data ptr   , vtable ptr)
Original fat ptr:(140314899668256, 4431298656)
C side handle here
C side send here
C side onSend here          (    data ptr   ,   vtable ptr   )
Fat ptr passed to callback: (140314899668256, 140732784668465)

```
You can see the ptr to the vtable is all messed up, like it didn't survive the trip.
Some more digging and I read somewhere that arguments for `extern "C"` functions
are passed via registers. I believe that is why I loose the vtable ptr in transit
since I only get the first 8 bytes.

To fix it, I just wrapped the `*mut dyn OnSend` a plain struct with `#[repr(C)]`
and passed that around. Now the code looks more like:
```rust
#[repr(C)]
struct RustSideHandler {
    opaque: *mut dyn OnSend,
}

#[repr(C)]
struct FFIWrapper {
    callback: extern "C" fn(*mut RustSideHandler, *const c_char, *const c_uchar, usize),
    //                       ^^^^^^^^^^^^^^^^^^^^ Changed from *mut dyn OnSend

    self_rust_side: *mut RustSideHandler, // Changed from *mut dyn OnSend
}

extern "C" fn handler_cb(
    rust_obj: *mut RustSideHandler, // Changed from *mut dyn OnSend
    dest: *const c_char,
    arg: *const c_uchar,
    arg_len: usize,
) {
    unsafe {
        let dest = CStr::from_ptr(dest);
        let sl = std::slice::from_raw_parts(arg, arg_len);

        println!("Fat ptr passed to callback: {:?}",
                 unsafe { transmute::<_, (usize, usize)>((*rust_obj).opaque) });

        let mut bv = std::boxed::Box::from_raw((*rust_obj).opaque);
        bv.as_mut().on_send(dest.to_str().unwrap(), sl); // <-- happy now
    }
}


pub fn handler_(dest: &str, handle: Box<dyn OnSend>) -> UserSpaceWrapper { /*hidden*/ }
    //... largely the same code,
    // now builds a RustSideHandler and places handle that inside the FFIWrapper

```
The output of the new code looked like
```shell
Original fat ptr:(140362144308512, 4348330080)
C side handle here
C side send here
C side onSend here
Fat ptr passed to callback: (140362144308512, 4348330080)
```
YAY :see_no_evil:

Bottom line, avoid passing fat pointers on extern calls OR maybe this whole thing
of passing a `*mut dyn OnSend` is a bad idea. I figured that shouldn't be that bad for
my use case since that `RustSideHandler` struct will NOT be accessed on the library side.