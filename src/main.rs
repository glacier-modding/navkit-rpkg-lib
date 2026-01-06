use glacier2obj::run_extraction;
use std::env;
use std::ffi::CStr;
use std::os::raw::c_char;

pub fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 6 {
        eprintln!("Usage: cargo run <path to a Retail directory> <game version (H2016 | HM2 | HM3)> <path to nav.json file> <path to a Runtime directory> <path to output directory> <output type (ALOC | PRIM)>");
        return;
    }

    run_extraction(args, log_callback);
}

extern "C" fn log_callback(msg: *const c_char) {
    if !msg.is_null() {
        unsafe {
            if let Ok(c_str) = CStr::from_ptr(msg).to_str() {
                println!("{}", c_str);
            }
        }
    }
}
