use cffi_explore;
use cffi_explore::{LibDummy, UserSpaceWrapper};
use std::sync::{Arc, RwLock};
use std::{thread, time};

const CIEN_MILLIS: time::Duration = time::Duration::from_millis(100);

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

fn setup_handler(lib: &LibDummy) -> (UserSpaceWrapper, Arc<RwLock<Option<String>>>) {
    let d = Arc::new(RwLock::new(None));
    let user = Box::new(UserSpaceHandler {
        val: Arc::clone(&d),
    });
    (lib.handler("here12", user), d)
}

fn wait_on_result(msg_rcvd: &Arc<RwLock<Option<String>>>) {
    let mut count = 0;
    loop {
        match msg_rcvd.try_read() {
            Ok(lck) if lck.is_some() => break,
            _ => (),
        }

        if count > 10 {
            panic!("failed to receive data");
        }
        count += 1;
        thread::sleep(CIEN_MILLIS);
    }
}

#[test]
fn start_lib() {
    let lib = cffi_explore::start_lib().unwrap();

    {
        let (_user, msg_rcvd) = setup_handler(&lib);
        let s = String::from("ledata to send");
        lib.send("here12", s.as_bytes());
        wait_on_result(&msg_rcvd);
        let msg = &*msg_rcvd.read().unwrap();
        assert_eq!(&s, msg.as_ref().unwrap())
    } //This scope is important so the user gets dropped before the shutdown is called

    lib.shutdown();
}
