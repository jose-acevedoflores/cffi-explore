use cffi_explore;
use std::{thread, time};
use std::cell::RefCell;
use std::rc::Rc;

struct UserSpaceHandler {
    v: Rc<RefCell<Option<String>>>,
}

impl cffi_explore::OnSend for UserSpaceHandler {
    fn on_send(&mut self, src: &str, arg: &[u8]) {
        println!("User space '{}'", src);
        // *(self.v.get_mut()) = Some(String::from_utf8(arg.to_vec()).unwrap());
        *self.v.borrow_mut() = Some(String::from_utf8(arg.to_vec()).unwrap());
    }
}

fn main() {
    let d = Rc::new(RefCell::new(None));
    let user = Box::new(UserSpaceHandler {
        v: Rc::clone(&d),
    });
    let h = cffi_explore::handler_("here",  user);

    let s = String::from("ledata to send");
    cffi_explore::send_("here", s.as_bytes());
    println!("We got it {:?}", &d);
    let two_secs = time::Duration::from_secs(2);
    thread::sleep(two_secs);
    cffi_explore::cancel_("here", h);
    thread::sleep(two_secs);
}
