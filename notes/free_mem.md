## Checking memory leaks!

todo

```rust
        //let mut bv = std::boxed::Box::from_raw((*rust_obj).opaque);
        //bv.as_mut().on_send(dest.to_str().unwrap(), sl); // <-- happy now
        //This is important!!
        // If we let the box 'bv' go out of scope it will free the contained 'dyn OnSend'
        // and next time it gets here, it will double free and segfault.
        //std::boxed::Box::into_raw(bv);
        // NOTE: after some thought, there's no need to rebuild the box
        // to then consume it again, we can just dereference the two pointers directly.
```
