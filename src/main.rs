use cffi_explore;
use std::sync::{Arc, RwLock};
use std::{thread, time};

struct UserSpaceHandler {
    val: Arc<RwLock<Option<String>>>,
}

impl cffi_explore::OnSend for UserSpaceHandler {
    fn on_send(&mut self, src: &str, arg: &[u8]) {
        let id = thread::current().id();
        println!("User space '{}' tid: {:?} ", src, id);
        let mut inner = self.val.write().unwrap();
        *inner = Some(String::from_utf8(arg.to_vec()).unwrap());
    }
}

fn main() {
    let d = Arc::new(RwLock::new(None));
    let user = Box::new(UserSpaceHandler {
        val: Arc::clone(&d),
    });
    let h = cffi_explore::handler_("here", user);

    let s = String::from("ledata to send");
    cffi_explore::send_("here", s.as_bytes());
    println!("We got it {:?}", &d);
    let two_secs = time::Duration::from_secs(2);
    thread::sleep(two_secs);
    cffi_explore::send_("here", "another one".as_bytes());
    println!("We got it {:?}", &d);
    let mut count = 0;
    while count < 2 {
        count += 1;
        thread::sleep(two_secs);
        println!("We got it {:?}", &d);
    }
    cffi_explore::cancel_("here", h);
    thread::sleep(two_secs);
    cffi_explore::shutdown_();
}
