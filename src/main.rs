use cffi_explore;
use cffi_explore::{LibDummy, UserSpaceWrapper};
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

fn setup_other_handler(lib: &LibDummy) -> UserSpaceWrapper {
    let d = Arc::new(RwLock::new(None));
    let user = Box::new(UserSpaceHandler {
        val: Arc::clone(&d),
    });
    lib.handler("here12", user)
}

fn main() {
    let lib = cffi_explore::start_lib().unwrap();
    let d = Arc::new(RwLock::new(None));
    let user = Box::new(UserSpaceHandler {
        val: Arc::clone(&d),
    });
    let h = lib.handler("here", user);

    let s = String::from("ledata to send");
    lib.send("here", s.as_bytes());
    println!("We got it {:?}", &d);
    let two_secs = time::Duration::from_secs(2);
    thread::sleep(two_secs);
    {
        let _h2 = setup_other_handler(&lib);
        lib.send("here", "another one".as_bytes());
        lib.send("here12", "for other handle".as_bytes());
        println!("We got it {:?}", &d);
        let mut count = 0;
        while count < 2 {
            count += 1;
            thread::sleep(two_secs);
            println!("We got it {:?}", &d);
        }
        lib.cancel("here", h);

        //NOTE: _h2 drops out here. The implicit drop will result
        // in a call to cancel which will segfault IF it's called after shutdown.
        // See notes in UserSpaceWrapper.drop
    }
    thread::sleep(two_secs);
    lib.shutdown();
}
