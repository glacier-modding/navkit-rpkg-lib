use navkit_rpkg_lib::package::package_scan::PackageScan;
use rpkg_rs::resource::runtime_resource_id::RuntimeResourceID;
use std::ffi::{c_char, CStr};

extern "C" fn log(msg: *const c_char) {
    println!("{}", unsafe { CStr::from_ptr(msg) }.to_string_lossy());
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let pm = PackageScan::scan_packages(args[1].clone(), "HM3".to_string(), log).expect("scan");
    for hash in &args[2..] {
        let rrid = RuntimeResourceID::from_hex_string(hash).expect("hash");
        println!(
            "{} -> {:?}",
            hash,
            navkit_rpkg_lib::resource_info_json(&pm, &rrid)
        );
    }
}
