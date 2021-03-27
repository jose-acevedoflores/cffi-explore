use cffi_explore;
use cffi_explore::{LibDummy, UserSpaceWrapper};
use env_logger::{Builder, Target};
use log::info;
use std::sync::{Arc, RwLock};
use std::{thread, time};

struct UserSpaceHandler {
    val: Arc<RwLock<Option<String>>>,
}

impl cffi_explore::OnSend for UserSpaceHandler {
    fn on_send(&self, src: &str, arg: &[u8]) {
        let id = thread::current().id();
        info!("User space '{}' tid: {:?} ", src, id);
        let mut inner = self.val.write().unwrap();
        *inner = Some(String::from_utf8(arg.to_vec()).unwrap());
    }

    fn on_send_inline(&self, src: &str, arg: &[u8]) -> Vec<u8> {
        let id = thread::current().id();
        info!("User space 'on_send_inline' - '{}' tid: {:?} ", src, id);

        let r = String::from_utf8(arg.to_vec()).unwrap();

        let r = format!("echoed: {}", r);
        r.into_bytes()
    }
}

fn setup_other_handler(lib: &LibDummy) -> UserSpaceWrapper {
    let d = Arc::new(RwLock::new(None));
    let user = Box::new(UserSpaceHandler {
        val: Arc::clone(&d),
    });
    lib.handler("here12", user).unwrap()
}

fn start_env_logger() {
    let mut env = Builder::from_default_env();
    env.target(Target::Stdout);
    env.format_timestamp_millis();
    env.init();
}

fn main() {
    start_env_logger();
    let lib = cffi_explore::start_lib().unwrap();
    let d = Arc::new(RwLock::new(None));
    let user = Box::new(UserSpaceHandler {
        val: Arc::clone(&d),
    });
    let h = lib.handler("here", user).unwrap();

    let s = String::from("ledata to send");
    lib.send("here", s.as_bytes());
    info!("We got it {:?}", &d);
    let two_secs = time::Duration::from_secs(2);
    thread::sleep(two_secs);
    {
        let _h2 = setup_other_handler(&lib);
        lib.send("here", "another one".as_bytes());
        lib.send("here12", "for other handle".as_bytes());
        info!("We got it {:?}", &d);
        let mut count = 0;
        while count < 2 {
            count += 1;
            thread::sleep(two_secs);
            info!("We got it {:?}", &d);
        }

        let vec = lib.send_inline("here", String::from("asd").as_bytes());
        let res = String::from_utf8(vec).unwrap();
        info!("Got Result Inline - '{}'!!", res);

        lib.cancel("here", h);

        thread::sleep(two_secs);

        #[cfg(feature = "with_lib_checks")]
        lib.shutdown();

        //NOTE: _h2 drops out here. The implicit drop will result
        // in a call to cancel AFTER the shutdown shutdown.
        // This should NOT cause a segfault.
    }

    //NOTE: the conditional compilation of 'lib.shutdown' is because without the lib checks, when _h2
    // is dropped it assumes lib is still valid. So, if we call 'lib.shutdown' in the scope above
    // when _h2 gets dropped it will segfault.

    #[cfg(not(feature = "with_lib_checks"))]
    lib.shutdown();
}
