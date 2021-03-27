pub fn init_log() {
    let _ = env_logger::builder().is_test(true).try_init();
}
