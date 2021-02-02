use cffi_explore;
use std::{thread, time};

fn main() {
    cffi_explore::handler_("here");

    let s = String::from("ledata to send");
    cffi_explore::send_("here", s.as_bytes());

    let ten_millis = time::Duration::from_secs(5);

    thread::sleep(ten_millis);
}
