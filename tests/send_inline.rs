use cffi_explore;
use cffi_explore::{LibDummy, UserSpaceWrapper};
use std::sync::{Arc, RwLock};
use std::{thread, time};

const HANDLER_FOR_TEST: &str = "here42";
const ECHO_PREFIX: &str = "echoed: ";

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

    fn on_send_inline(&mut self, src: &str, arg: &[u8]) -> Vec<u8> {
        let id = thread::current().id();
        println!("User space 'on_send_inline' - '{}' tid: {:?} ", src, id);

        let r = String::from_utf8(arg.to_vec()).unwrap();
        let r = format!("{}{}", ECHO_PREFIX, r);
        r.into_bytes()
    }
}

fn setup_handler(lib: &LibDummy) -> (UserSpaceWrapper, Arc<RwLock<Option<String>>>) {
    let d = Arc::new(RwLock::new(None));
    let user = Box::new(UserSpaceHandler {
        val: Arc::clone(&d),
    });
    (lib.handler(HANDLER_FOR_TEST, user), d)
}

#[test]
fn test_send_inline() {
    let lib = cffi_explore::start_lib().unwrap();
    {
        let (_user, _msg_rcvd) = setup_handler(&lib);
        let s = String::from("ledata to send");
        let vec = lib.send_inline(HANDLER_FOR_TEST, s.as_bytes());
        let echoed_string = String::from_utf8(vec).unwrap();
        // assert_eq!(format!("{}{}", ECHO_PREFIX, s), echoed_string);
    }
    lib.shutdown();
}
