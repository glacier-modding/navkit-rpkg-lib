extern crate core;

pub mod extract;
pub mod json_serde;
pub mod package;

use crate::extract::rpkg_extraction::RpkgExtraction;
use crate::json_serde::entities_json::EntitiesJson;
use crate::package::package_scan::PackageScan;
use std::collections::HashSet;
use std::ffi::CStr;
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn run_extraction_wrapper(
    retail_directory: *const c_char,
    game_version: *const c_char,
    nav_json_file: *const c_char,
    runtime_directory: *const c_char,
    output_directory: *const c_char,
    output_type: *const c_char,
    log_callback: extern "C" fn(*const c_char),
) -> std::ffi::c_int {
    let args = vec![
        "glacier2obj".to_string(),
        unsafe {
            CStr::from_ptr(retail_directory)
                .to_string_lossy()
                .into_owned()
        },
        unsafe { CStr::from_ptr(game_version).to_string_lossy().into_owned() },
        unsafe { CStr::from_ptr(nav_json_file).to_string_lossy().into_owned() },
        unsafe {
            CStr::from_ptr(runtime_directory)
                .to_string_lossy()
                .into_owned()
        },
        unsafe {
            CStr::from_ptr(output_directory)
                .to_string_lossy()
                .into_owned()
        },
        unsafe { CStr::from_ptr(output_type).to_string_lossy().into_owned() },
    ];

    run_extraction(args, log_callback)
}

pub fn run_extraction(
    args: Vec<String>,
    log_callback: extern "C" fn(*const c_char),
) -> std::ffi::c_int {
    if args.len() < 7 {
        // args[0] is program name
        let msg = std::ffi::CString::new("Usage: cargo run <path to a Retail directory> <game version (H2016 | HM2 | HM3)> <path to nav.json file> <path to a Runtime directory> <path to output directory> <output type (ALOC | PRIM)>").unwrap();
        log_callback(msg.as_ptr());
        return -1;
    }

    let msg = std::ffi::CString::new("glacier2obj - Starting extraction from RPKG files.").unwrap();
    log_callback(msg.as_ptr());

    let nav_json = match EntitiesJson::build_from_nav_json_file(args[3].clone(), log_callback) {
        Some(json) => json,
        None => {
            return -1;
        }
    };
    let needed_aloc_or_prim_hashes = RpkgExtraction::get_all_aloc_or_prim_hashes_from_scene(
        &nav_json,
        args[5].clone(),
        args[6].clone(),
        log_callback,
    );
    let mut result: std::ffi::c_int = 0;

    if needed_aloc_or_prim_hashes.is_empty() {
        let msg =
            std::ffi::CString::new("All aloc or prim files already exist. Skipping extraction.")
                .unwrap();
        log_callback(msg.as_ptr());
    } else {
        if args[6].clone() == "ALOC" {
            let msg = std::ffi::CString::new(format!(
                "Extracting {} alocs.",
                needed_aloc_or_prim_hashes.len()
            ))
            .unwrap();
            log_callback(msg.as_ptr());
        } else if args[6].clone() == "PRIM" {
            let msg = std::ffi::CString::new(format!(
                "Extracting {} prims.",
                needed_aloc_or_prim_hashes.len()
            ))
            .unwrap();
            log_callback(msg.as_ptr());
        } else {
            let msg = std::ffi::CString::new(format!(
                "Error - Output type '{}' must be ALOC or PRIM.",
                args[6].clone()
            ))
            .unwrap();
            log_callback(msg.as_ptr());
            return -1;
        }

        let partition_manager =
            match PackageScan::scan_packages(args[1].clone(), args[2].clone(), log_callback) {
                Some(manager) => manager,
                None => {
                    return -1;
                }
            };
        result = RpkgExtraction::extract_resources_from_rpkg(
            args[4].clone(),
            needed_aloc_or_prim_hashes,
            &partition_manager,
            args[5].clone(),
            args[6].clone(),
            log_callback,
        );
    }
    let msg = std::ffi::CString::new("Done extracting alocs or prims.").unwrap();
    log_callback(msg.as_ptr());
    result
}

#[no_mangle]
pub extern "C" fn build_from_nav_json_file(
    nav_json_file: *const c_char,
    log_callback: extern "C" fn(*const c_char),
) -> *mut EntitiesJson {
    let nav_json_file_str = unsafe { CStr::from_ptr(nav_json_file).to_string_lossy().into_owned() };
    let entities_json = match EntitiesJson::build_from_nav_json_file(nav_json_file_str, log_callback) {
        Some(json) => json,
        None => {
            return std::ptr::null_mut();
        }
    };
    Box::into_raw(Box::new(entities_json))
}

#[no_mangle]
pub extern "C" fn free_entities_json(ptr: *mut EntitiesJson) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(ptr);
    }
}

#[no_mangle]
pub extern "C" fn get_all_aloc_or_prim_hashes(
    entities_json: *const EntitiesJson,
    output_folder: *const c_char,
    output_type: *const c_char,
    log_callback: extern "C" fn(*const c_char),
) -> *mut HashSet<String> {
    let entities_json_ref = unsafe { &*entities_json };
    let output_folder_str = unsafe { CStr::from_ptr(output_folder).to_string_lossy().into_owned() };
    let output_type_str = unsafe { CStr::from_ptr(output_type).to_string_lossy().into_owned() };

    let hashes = RpkgExtraction::get_all_aloc_or_prim_hashes_from_scene(
        entities_json_ref,
        output_folder_str,
        output_type_str,
        log_callback,
    );
    Box::into_raw(Box::new(hashes))
}

#[no_mangle]
pub extern "C" fn free_hashset_string(ptr: *mut HashSet<String>) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(ptr);
    }
}

#[no_mangle]
pub extern "C" fn scan_packages(
    retail_folder: *const c_char,
    game_version: *const c_char,
    log_callback: extern "C" fn(*const c_char),
) -> *mut rpkg_rs::resource::partition_manager::PartitionManager {
    let retail_folder_str = unsafe { CStr::from_ptr(retail_folder).to_string_lossy().into_owned() };
    let game_version_str = unsafe { CStr::from_ptr(game_version).to_string_lossy().into_owned() };

    let pm = PackageScan::scan_packages(retail_folder_str, game_version_str, log_callback);
    match pm {
        Some(manager) => Box::into_raw(Box::new(manager)),
        None => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn free_partition_manager(
    ptr: *mut rpkg_rs::resource::partition_manager::PartitionManager,
) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(ptr);
    }
}

#[no_mangle]
pub extern "C" fn extract_resources_from_rpkg(
    runtime_folder: *const c_char,
    needed_hashes: *mut HashSet<String>,
    partition_manager: *const rpkg_rs::resource::partition_manager::PartitionManager,
    output_folder: *const c_char,
    resource_type: *const c_char,
    log_callback: extern "C" fn(*const c_char),
) {
    let runtime_folder_str = unsafe {
        CStr::from_ptr(runtime_folder)
            .to_string_lossy()
            .into_owned()
    };
    let needed_hashes_ref = unsafe { &*needed_hashes }; // We clone it because extract_alocs_or_prims takes ownership or we change signature
                                                        // The original extract_alocs_or_prims takes HashSet<String> by value.
                                                        // So we need to clone it here if we want to keep the pointer valid, or take ownership.
                                                        // Let's clone for safety so the C++ side still owns the set until it frees it.
    let needed_hashes_clone = needed_hashes_ref.clone();

    let partition_manager_ref = unsafe { &*partition_manager };
    let output_folder_str = unsafe { CStr::from_ptr(output_folder).to_string_lossy().into_owned() };
    let resource_type_str = unsafe { CStr::from_ptr(resource_type).to_string_lossy().into_owned() };

    RpkgExtraction::extract_resources_from_rpkg(
        runtime_folder_str,
        needed_hashes_clone,
        partition_manager_ref,
        output_folder_str,
        resource_type_str,
        log_callback,
    );
}
