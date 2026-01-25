extern crate core;

pub mod extract;
pub mod json_serde;
pub mod package;

use crate::extract::rpkg_extraction::RpkgExtraction;
use crate::json_serde::entities_json::EntitiesJson;
use crate::package::package_scan::PackageScan;
use std::collections::HashSet;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn extract_scene_mesh_resources(
    nav_json_file: *const c_char,
    runtime_directory: *const c_char,
    partition_manager: *const rpkg_rs::resource::partition_manager::PartitionManager,
    output_directory: *const c_char,
    output_type: *const c_char,
    log_callback: extern "C" fn(*const c_char),
) -> std::ffi::c_int {
    let output_directory_str = unsafe {
        CStr::from_ptr(output_directory)
            .to_string_lossy()
            .into_owned()
    };
    let output_type_str = unsafe { CStr::from_ptr(output_type).to_string_lossy().into_owned() };

    let msg = CString::new("navkit-rpkg-lib - Starting extraction from RPKG files.").unwrap();
    log_callback(msg.as_ptr());

    let runtime_directory_ref = unsafe {
        CStr::from_ptr(runtime_directory)
            .to_string_lossy()
            .into_owned()
    };
    let nav_json = match EntitiesJson::build_from_nav_json_file(
        unsafe { CStr::from_ptr(nav_json_file).to_string_lossy().into_owned() }.clone(),
        log_callback,
    ) {
        Some(json) => json,
        None => {
            return -1;
        }
    };
    let needed_aloc_or_prim_hashes = RpkgExtraction::get_needed_aloc_or_prim_hashes_from_scene(
        &nav_json,
        output_type_str.clone(),
    );
    let mut result: std::ffi::c_int = 0;

    if needed_aloc_or_prim_hashes.is_empty() {
        let msg = CString::new(format!(
            "All {} files already exist. Skipping extraction.",
            output_type_str
        ))
        .unwrap();
        log_callback(msg.as_ptr());
        return result;
    }
    let msg = CString::new(format!(
        "Extracting {} {}s.",
        needed_aloc_or_prim_hashes.len(),
        output_type_str
    ))
    .unwrap();
    log_callback(msg.as_ptr());

    let needed_aloc_or_prim_hash_strings: Vec<std::ffi::CString> = needed_aloc_or_prim_hashes
        .iter()
        .map(|s| std::ffi::CString::new(s.as_str()).unwrap())
        .collect();
    let needed_aloc_or_prim_hashes_c_ptrs: Vec<*const c_char> = needed_aloc_or_prim_hash_strings
        .iter()
        .map(|c| c.as_ptr())
        .collect();

    let partition_manager_ref = unsafe { &*partition_manager };
    unsafe {
        result = RpkgExtraction::extract_resources_from_rpkg(
            runtime_directory_ref.clone(),
            needed_aloc_or_prim_hashes_c_ptrs.as_ptr(),
            needed_aloc_or_prim_hashes_c_ptrs.len(),
            partition_manager_ref,
            output_directory_str.clone(),
            output_type_str.clone(),
            log_callback,
        );
    }
    result
}

#[no_mangle]
pub extern "C" fn extract_resources_from_rpkg(
    runtime_folder: *const c_char,
    needed_hashes: *const *const c_char,
    needed_hashes_len: usize,
    partition_manager: *const rpkg_rs::resource::partition_manager::PartitionManager,
    output_folder: *const c_char,
    resource_type: *const c_char,
    log_callback: extern "C" fn(*const c_char),
) {
    let msg = std::ffi::CString::new("Extracting Resources from rpkg.".to_string()).unwrap();
    log_callback(msg.as_ptr());
    let runtime_folder_str = unsafe {
        CStr::from_ptr(runtime_folder)
            .to_string_lossy()
            .into_owned()
    };

    let partition_manager_ref = unsafe { &*partition_manager };
    let output_folder_str = unsafe { CStr::from_ptr(output_folder).to_string_lossy().into_owned() };
    let resource_type_str = unsafe { CStr::from_ptr(resource_type).to_string_lossy().into_owned() };
    unsafe {
        RpkgExtraction::extract_resources_from_rpkg(
            runtime_folder_str,
            needed_hashes,
            needed_hashes_len,
            partition_manager_ref,
            output_folder_str,
            resource_type_str,
            log_callback,
        );
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
pub extern "C" fn get_all_resources_hashes_by_type_from_rpkg_files(
    partition_manager: *const rpkg_rs::resource::partition_manager::PartitionManager,
    resource_type: *const c_char,
    log_callback: extern "C" fn(*const c_char),
) -> *mut RustStringList {
    let resource_type_str = unsafe { CStr::from_ptr(resource_type).to_string_lossy().into_owned() };
    let msg = std::ffi::CString::new(
        format!(
            "Scanning Rpkg files for all {} Resources.",
            resource_type_str
        )
        .to_string(),
    )
    .unwrap();
    log_callback(msg.as_ptr());

    let partition_manager_ref = unsafe { &*partition_manager };
    let resources = RpkgExtraction::get_all_resources_hashes_by_type_from_rpkg_files(
        partition_manager_ref,
        resource_type_str,
        log_callback,
    );
    create_string_list(resources)
}

#[no_mangle]
pub extern "C" fn get_hash_list_from_file_or_repo(
    output_folder: *const c_char,
    log_callback: extern "C" fn(*const c_char),
) -> *mut hitman_commons::hash_list::HashList {
    let output_folder_str = unsafe { CStr::from_ptr(output_folder).to_string_lossy().into_owned() };

    let hash_list = RpkgExtraction::get_hash_list_from_file_or_repo(output_folder_str, log_callback);
    match hash_list {
        Ok(hash_list) => Box::into_raw(Box::new(hash_list)),
        Err => std::ptr::null_mut(),
    }
}

#[repr(C)]
pub struct RustStringList {
    entries: *mut *mut c_char,
    length: usize,
}

pub fn create_string_list(strings: Vec<String>) -> *mut RustStringList {
    let rust_strings = strings
        .into_iter()
        .filter_map(|s| CString::new(s).ok())
        .map(|c_string| c_string.into_raw())
        .collect::<Vec<*mut c_char>>();

    let length = rust_strings.len();
    let entries = rust_strings.leak().as_mut_ptr();

    let list = Box::new(RustStringList { entries, length });
    Box::into_raw(list)
}

#[no_mangle]
pub extern "C" fn get_string_from_list(list: *mut RustStringList, index: usize) -> *const c_char {
    if list.is_null() {
        return std::ptr::null();
    }
    unsafe {
        let list_ref = &*list;
        if index >= list_ref.length {
            return std::ptr::null();
        }
        // Access the pointer at the given index
        *list_ref.entries.add(index)
    }
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
pub extern "C" fn free_hashset_string(ptr: *mut HashSet<String>) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(ptr);
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
