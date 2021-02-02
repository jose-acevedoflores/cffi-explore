fn main() {
    let cwd = std::env::current_dir().unwrap();
    let lib_path = cwd.join("libdummy");
    println!("cargo:rustc-link-search={}", lib_path.to_str().unwrap());
}

/* NOTE:

Won't link .so without
When running, don't forget to set LD_LIBRARY_PATH or run with:
DYLD_LIBRARY_PATH=<path/to/>/cffi-explore/libdummy cargo <run|test|build>


 */
