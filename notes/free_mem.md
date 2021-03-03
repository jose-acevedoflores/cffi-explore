## Checking memory leaks!

Since I'm so used to GCd languages, I thought it might be a good idea to document the steps I took
to check if my wrapper library was properly managing memory. In this (completely unrelated article)
[article](https://fasterthanli.me/articles/so-you-want-to-live-reload-rust)
I found out about `valgrind` and how it can help detect memory leaks in a program.

I proceeded to run the `valgrind` on my binary and ***voilà***:

```shell
==30788== HEAP SUMMARY:
==30788==     in use at exit: 235 bytes in 7 blocks
==30788==   total heap usage: 34 allocs, 27 frees, 77,412 bytes allocated
==30788==
==30788==
==30788== LEAK SUMMARY:
==30788==    definitely lost: 16 bytes in 1 blocks
==30788==    indirectly lost: 83 bytes in 4 blocks
==30788==      possibly lost: 0 bytes in 0 blocks
==30788==    still reachable: 136 bytes in 2 blocks
==30788==         suppressed: 0 bytes in 0 blocks
```
IT'S LEAKING.

I figured it had to be how I was consuming the Boxed values inside the `handler_` function,
and preventing the Drop from ever running on those boxes.

The handle/cancel code looked like:
```rust
pub fn handler_(dest: &str, handle: Box<dyn OnSend>) -> UserSpaceWrapper {
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
    res >= 0
}

```

A simple addition to retake the boxes and let the drop sufficed.
Now the code looks like:
```rust
pub fn handler_(dest: &str, handle: Box<dyn OnSend>) -> UserSpaceWrapper {
    // .... body same as before
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
```

I ran `valgrind` again after the changes and:
```shell
==12249== HEAP SUMMARY:
==12249==     in use at exit: 0 bytes in 0 blocks
==12249==   total heap usage: 34 allocs, 34 frees, 77,412 bytes allocated
==12249==
==12249== All heap blocks were freed -- no leaks are possible
==12249==
==12249== For lists of detected and suppressed errors, rerun with: -s
==12249== ERROR SUMMARY: 0 errors from 0 contexts (suppressed: 0 from 0)
```

It's working great now!

####Misc

Even though I just wanted to focus on the rust side, there were some leaks on the
`libdummy` side. Specifically, how it allocated `auto lib = new MyLibrary();`
statically and never freed it. For this reason I added a shutdown method on the `c`
side because I wanted to have the satisfaction of having `valgrind` report 0 memory leaks.

- valgrind command used `valgrind --leak-check=full target/debug/cffi-explore`.
  NOTE: with the `cmake` built `libdummy` don't forget to set
  `LD_LIBRARY_PATH=/<abs path to>/cffi-explore/target/debug/deps/` in order
  for valgrind to find the `libdummy.so` otherwise you'll get this error

```
  target/debug/cffi-explore: error while loading shared libraries: libdummy.so: cannot open shared object file: No such file or directory
 ```