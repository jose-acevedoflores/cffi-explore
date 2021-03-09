use cffi_explore;
use cffi_explore::{LibDummy, UserSpaceWrapper};
use std::sync::{Arc, RwLock};
use std::{thread, time};

const CIEN_MILLIS: time::Duration = time::Duration::from_millis(100);
const HANDLER_FOR_TEST: &str = "here12";

struct UserSpaceHandler {
    val: Arc<RwLock<Option<String>>>,
}

impl cffi_explore::OnSend for UserSpaceHandler {
    fn on_send(&self, src: &str, arg: &[u8]) {
        let id = thread::current().id();
        println!("User space '{}' tid: {:?} ", src, id);
        let mut inner = self.val.write().unwrap();
        *inner = Some(String::from_utf8(arg.to_vec()).unwrap());
    }

    fn on_send_inline(&self, _src: &str, _arg: &[u8]) -> Vec<u8> {
        unimplemented!()
    }
}

fn setup_handler(lib: &LibDummy) -> (UserSpaceWrapper, Arc<RwLock<Option<String>>>) {
    let d = Arc::new(RwLock::new(None));
    let user = Box::new(UserSpaceHandler {
        val: Arc::clone(&d),
    });
    (lib.handler(HANDLER_FOR_TEST, user).unwrap(), d)
}

fn wait_on_result(msg_rcvd: &Arc<RwLock<Option<String>>>) {
    let mut loop_cnt = 0;
    let mut num_calls = 0;
    loop {
        match msg_rcvd.try_read() {
            Ok(lck) if lck.is_some() => {
                //Expect two calls, one on the same thread and one in the bg thread.
                if num_calls >= 2 {
                    break;
                }
                num_calls += 1;
            }
            _ => (),
        }

        if loop_cnt > 100 {
            panic!("failed to receive data");
        }
        loop_cnt += 1;
        thread::sleep(CIEN_MILLIS);
    }
}

#[test]
fn start_lib() {
    let lib = cffi_explore::start_lib().unwrap();

    {
        let (_user, msg_rcvd) = setup_handler(&lib);
        let s = String::from("ledata to send");
        lib.send(HANDLER_FOR_TEST, s.as_bytes());
        wait_on_result(&msg_rcvd);
        let msg = &*msg_rcvd.read().unwrap();
        assert_eq!(&s, msg.as_ref().unwrap())
    } //This scope is important so the user gets dropped before the shutdown is called

    lib.shutdown();
}
