use cffi_explore;
use std::{thread, time};

struct UserSpaceHandler {
    arg_cp: Option<String>
}

impl cffi_explore::OnSend for UserSpaceHandler {
    fn on_send(&mut self, src: &str, arg: &[u8]) {
        println!("User space '{}'", src);
        self.arg_cp = Some(String::from_utf8(arg.to_vec()).unwrap());
    }
}

fn main() {
    let mut user = UserSpaceHandler {arg_cp: None};
    let h = cffi_explore::handler_("here", &mut user);

    let s = String::from("ledata to send");
    cffi_explore::send_("here", s.as_bytes());

    println!("We got it {:?}", user.arg_cp);

    let ten_millis = time::Duration::from_secs(5);

    thread::sleep(ten_millis);
}
