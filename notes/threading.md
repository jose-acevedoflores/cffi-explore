## Threading!

The last step in the `libdummy` wrapper would be to provide threading guards since the
hypothetical `libdummy` callbacks will happen in their own threads spawned on the `c` side.
By changing the signature from:
```rust
pub fn handler_(dest: &str, handle: Box<dyn OnSend>) -> UserSpaceWrapper {}
```
to
```rust
pub fn handler_(dest: &str, handle: Box<dyn OnSend + Sync>) -> UserSpaceWrapper {}
```

I can convey to the user of the library that any struct passed to the `handler_` needs to
properly manage being accessed by multiple threads.

NOTE: This seems deceptively simple so, I'm still wrapping my head around it.