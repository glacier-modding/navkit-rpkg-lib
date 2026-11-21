extern crate core;

pub mod extract;
pub mod json_serde;
pub mod package;

use crate::extract::rpkg_extraction::RpkgExtraction;
use crate::json_serde::entities_json::EntitiesJson;
use crate::package::package_scan::PackageScan;
use glacier_commons::hash_list::HashList;
use glacier_commons::metadata::RuntimeID;
use rpkg_rs::resource::partition_manager::PartitionManager;
use rpkg_rs::resource::runtime_resource_id::RuntimeResourceID;
use std::collections::HashSet;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::str::FromStr;

#[no_mangle]
pub extern "C" fn extract_scene_mesh_resources(
    nav_json_file: *const c_char,
    runtime_directory: *const c_char,
    partition_manager: *const PartitionManager,
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
    partition_manager: *const PartitionManager,
    output_folder: *const c_char,
    resource_type: *const c_char,
    log_callback: extern "C" fn(*const c_char),
) {
    let msg = CString::new("Extracting Resources from rpkg.".to_string()).unwrap();
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
) -> *mut PartitionManager {
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
    let msg = CString::new(
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
) -> *mut HashList {
    let output_folder_str = unsafe { CStr::from_ptr(output_folder).to_string_lossy().into_owned() };

    let hash_list =
        RpkgExtraction::get_hash_list_from_file_or_repo(output_folder_str, log_callback);
    Box::into_raw(Box::new(hash_list.unwrap()))
}

#[no_mangle]
pub extern "C" fn get_all_referenced_hashes_by_hash_from_rpkg_files(
    resource_hash: *const c_char,
    partition_manager: *const PartitionManager,
    log_callback: extern "C" fn(*const c_char),
) -> *mut RustStringList {
    // let msg = CString::new("Converting resource_hash_str in Rpkg files.".to_string()).unwrap();
    // log_callback(msg.as_ptr());
    let resource_hash_str = unsafe { CStr::from_ptr(resource_hash).to_string_lossy().into_owned() };
    // let msg = CString::new("Converting partition_manager_ref in Rpkg files.".to_string()).unwrap();
    // log_callback(msg.as_ptr());
    let partition_manager_ref = unsafe { &*partition_manager };
    // let msg = CString::new("Getting references in Rpkg files.".to_string()).unwrap();
    // log_callback(msg.as_ptr());

    let references = match RpkgExtraction::get_all_referenced_hashes_by_hash_from_rpkg_files(
        resource_hash_str,
        partition_manager_ref,
        log_callback,
    ) {
        Ok(references) => Some(references),
        Err(_) => return std::ptr::null_mut(),
    };
    create_string_list(references.unwrap())
}

#[no_mangle]
pub extern "C" fn get_mati_json_by_hash(
    resource_hash: *const c_char,
    partition_manager: *const PartitionManager,
    log_callback: extern "C" fn(*const c_char),
) -> *mut c_char {
    let resource_hash_str = unsafe { CStr::from_ptr(resource_hash).to_string_lossy().into_owned() };
    let partition_manager_ref = unsafe { &*partition_manager };

    match RpkgExtraction::get_mati_json_by_hash(
        resource_hash_str,
        partition_manager_ref,
        log_callback,
    ) {
        Ok(json) => CString::new(json).unwrap_or_default().into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

fn option_string_to_c_char(s: Option<String>) -> *mut c_char {
    match s {
        Some(str) => CString::new(str).map_or(std::ptr::null_mut(), |c| c.into_raw()),
        None => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn hash_list_get_version(list: *const HashList) -> u32 {
    if list.is_null() {
        return 0;
    }
    let list_ref = unsafe { &*list };
    list_ref.version.load(std::sync::atomic::Ordering::Relaxed)
}

#[no_mangle]
pub extern "C" fn hash_list_get_all_hashes(list: *const HashList) -> *mut RustStringList {
    if list.is_null() {
        // Return an empty list if the hash list is null
        return create_string_list(vec![]);
    }
    let list_ref = unsafe { &*list };
    let map_guard = list_ref.entries.load();

    let hashes: Vec<String> = map_guard.keys().map(|rrid| rrid.to_string()).collect();
    create_string_list(hashes)
}

#[no_mangle]
pub extern "C" fn hash_list_get_path_by_hash(
    list: *const HashList,
    resource_hash: *const c_char,
) -> *mut c_char {
    if list.is_null() || resource_hash.is_null() {
        return std::ptr::null_mut();
    }
    let list_ref = unsafe { &*list };
    let hash_str = unsafe { CStr::from_ptr(resource_hash).to_string_lossy() };

    if let Ok(rrid) = RuntimeID::from_str(&hash_str) {
        if let Some(data) = list_ref.entries.load().get(&rrid) {
            if let Some(path) = data.path.as_ref() {
                return option_string_to_c_char(Some(String::from(path.clone())));
            }
        }
    }
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn hash_list_get_hint_by_hash(
    list: *const HashList,
    resource_hash: *const c_char,
) -> *mut c_char {
    if list.is_null() || resource_hash.is_null() {
        return std::ptr::null_mut();
    }
    let list_ref = unsafe { &*list };
    let hash_str = unsafe { CStr::from_ptr(resource_hash).to_string_lossy() };

    if let Ok(rrid) = RuntimeID::from_str(&hash_str) {
        if let Some(data) = list_ref.entries.load().get(&rrid) {
            if let Some(hint) = data.hint.as_ref() {
                return option_string_to_c_char(Some(String::from(hint.clone())));
            }
        }
    }
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn hash_list_get_resource_type_by_hash(
    list: *const HashList,
    resource_hash: *const c_char,
) -> u32 {
    // NOTE: This assumes `ResourceType` has an `Unknown` variant and is `#[repr(C)]`.
    // If not, this function will need to return an i32/u32 instead.
    const UNKNOWN_TYPE_CODE: u32 = 0;
    if list.is_null() || resource_hash.is_null() {
        return UNKNOWN_TYPE_CODE;
    }
    let list_ref = unsafe { &*list };
    let hash_str = unsafe { CStr::from_ptr(resource_hash).to_string_lossy() };

    if let Ok(rrid) = RuntimeID::from_str(&hash_str) {
        if let Some(data) = list_ref.entries.load().get(&rrid) {
            let bytes: [u8; 4] = data.resource_type.into();
            return u32::from_be_bytes(bytes);
        }
    }
    UNKNOWN_TYPE_CODE
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
pub extern "C" fn free_rust_string_list(ptr: *mut RustStringList) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let list = Box::from_raw(ptr);
        // Reconstruct the Vec of pointers to drop it, then free each string.
        let strings = Vec::from_raw_parts(list.entries, list.length, list.length);
        for s_ptr in strings {
            // This reconstructs the CString and allows its memory to be freed.
            let _ = CString::from_raw(s_ptr);
        }
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
pub extern "C" fn free_string(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let _ = CString::from_raw(ptr);
    }
}

#[no_mangle]
pub extern "C" fn free_hash_list(ptr: *mut glacier_commons::hash_list::HashList) {
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

/// Builds the JSON that `get_resource_info_json` returns for `rrid`, or `None` when no partition holds it.
pub fn resource_info_json(
    partition_manager: &PartitionManager,
    rrid: &RuntimeResourceID,
) -> Option<String> {
    let info = PackageScan::get_resource_info(partition_manager, rrid)?;
    let references: Vec<serde_json::Value> = info
        .last_occurrence
        .references()
        .iter()
        .map(|(reference, _)| {
            let reference_type = PackageScan::get_resource_info(partition_manager, reference)
                .map(|reference_info| reference_info.last_occurrence.data_type())
                .unwrap_or_default();
            serde_json::json!({ "hash": reference.to_hex_string(), "type": reference_type })
        })
        .collect();
    Some(
        serde_json::json!({
            "hash": rrid.to_hex_string(),
            "type": info.last_occurrence.data_type(),
            "partition": info.last_partition,
            "references": references,
        })
        .to_string(),
    )
}

/// Returns a JSON object describing the latest version of a resource:
/// `{"hash": "00...", "type": "TEMP", "partition": "chunk0patch3.rpkg", "references": [{"hash": "00...", "type": "TBLU"}]}`.
/// A reference that no partition holds has an empty type. Returns null when the hash is invalid or not found.
/// Free the result with `free_string`.
#[no_mangle]
pub extern "C" fn get_resource_info_json(
    resource_hash: *const c_char,
    partition_manager: *const PartitionManager,
    log_callback: extern "C" fn(*const c_char),
) -> *mut c_char {
    if resource_hash.is_null() || partition_manager.is_null() {
        return std::ptr::null_mut();
    }
    let resource_hash_str = unsafe { CStr::from_ptr(resource_hash).to_string_lossy().into_owned() };
    let rrid = match RuntimeResourceID::from_hex_string(resource_hash_str.as_str()) {
        Ok(rrid) => rrid,
        Err(_) => {
            let msg =
                CString::new(format!("Invalid resource hash: {}", resource_hash_str)).unwrap();
            log_callback(msg.as_ptr());
            return std::ptr::null_mut();
        }
    };
    let partition_manager_ref = unsafe { &*partition_manager };
    option_string_to_c_char(resource_info_json(partition_manager_ref, &rrid))
}

#[no_mangle]
pub extern "C" fn pack_gfx_to_gfxf(
    gfx_data: *const u8,
    gfx_data_len: usize,
    output_file_path: *const c_char,
    log_callback: extern "C" fn(*const c_char),
) -> std::ffi::c_int {
    if gfx_data.is_null() || output_file_path.is_null() {
        let msg = CString::new("Invalid input parameters: null pointer provided").unwrap();
        log_callback(msg.as_ptr());
        return -1;
    }

    let gfx_vec = unsafe { std::slice::from_raw_parts(gfx_data, gfx_data_len).to_vec() };
    let output_path_str = unsafe {
        CStr::from_ptr(output_file_path)
            .to_string_lossy()
            .into_owned()
    };

    let msg = CString::new(format!("Packing GFX data ({} bytes) into GFXF", gfx_data_len)).unwrap();
    log_callback(msg.as_ptr());

    let gfxf_resource = glacier_bin1::game::h3::SGFxMovieResource {
        gfx_data: gfx_vec,
        images_filename: Vec::new(),
        images_data: Vec::new(),
    };

    let serialized = match glacier_bin1::serialize(&gfxf_resource) {
        Ok(data) => data,
        Err(e) => {
            let msg = CString::new(format!("Failed to serialize GFXF: {}", e)).unwrap();
            log_callback(msg.as_ptr());
            return -1;
        }
    };

    let msg = CString::new(format!("Writing GFXF file to: {}", output_path_str)).unwrap();
    log_callback(msg.as_ptr());

    if let Err(e) = std::fs::write(&output_path_str, serialized) {
        let msg = CString::new(format!("Failed to write GFXF file: {}", e)).unwrap();
        log_callback(msg.as_ptr());
        return -1;
    }

    let msg = CString::new("Successfully packed GFX into GFXF and saved to file").unwrap();
    log_callback(msg.as_ptr());
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock log callback for testing
    extern "C" fn test_log_callback(_msg: *const c_char) {
        // No-op for tests
    }

    #[test]
    fn test_pack_gfx_to_gfxf_success() {
        let gfx_data = b"test-gfx-data";
        let temp_dir = std::env::temp_dir();
        let output_path = temp_dir.join("test_output.gfxf");
        let output_path_cstring = CString::new(output_path.to_str().unwrap()).unwrap();

        let result = pack_gfx_to_gfxf(
            gfx_data.as_ptr(),
            gfx_data.len(),
            output_path_cstring.as_ptr(),
            test_log_callback,
        );

        assert_eq!(result, 0);
        assert!(output_path.exists());

        // Verify the file contains valid GFXF data
        let file_contents = std::fs::read(&output_path).unwrap();
        let gfxf: glacier_bin1::game::h3::SGFxMovieResource =
            glacier_bin1::deserialize(&file_contents).unwrap();
        assert_eq!(gfxf.gfx_data, gfx_data);

        // Cleanup
        std::fs::remove_file(output_path).ok();
    }

    #[test]
    fn test_pack_gfx_to_gfxf_null_data() {
        let output_path = CString::new("/tmp/test.gfxf").unwrap();

        let result = pack_gfx_to_gfxf(
            std::ptr::null(),
            0,
            output_path.as_ptr(),
            test_log_callback,
        );

        assert_eq!(result, -1);
    }

    #[test]
    fn test_pack_gfx_to_gfxf_null_path() {
        let gfx_data = b"test-data";

        let result = pack_gfx_to_gfxf(
            gfx_data.as_ptr(),
            gfx_data.len(),
            std::ptr::null(),
            test_log_callback,
        );

        assert_eq!(result, -1);
    }

    #[test]
    fn test_create_string_list() {
        let strings = vec!["hash1".to_string(), "hash2".to_string(), "hash3".to_string()];
        let list_ptr = create_string_list(strings);

        assert!(!list_ptr.is_null());

        unsafe {
            let list = &*list_ptr;
            assert_eq!(list.length, 3);

            let str1 = CStr::from_ptr(get_string_from_list(list_ptr, 0)).to_str().unwrap();
            let str2 = CStr::from_ptr(get_string_from_list(list_ptr, 1)).to_str().unwrap();
            let str3 = CStr::from_ptr(get_string_from_list(list_ptr, 2)).to_str().unwrap();

            assert_eq!(str1, "hash1");
            assert_eq!(str2, "hash2");
            assert_eq!(str3, "hash3");
        }

        free_rust_string_list(list_ptr);
    }

    #[test]
    fn test_create_string_list_empty() {
        let strings = vec![];
        let list_ptr = create_string_list(strings);

        assert!(!list_ptr.is_null());

        unsafe {
            let list = &*list_ptr;
            assert_eq!(list.length, 0);
        }

        free_rust_string_list(list_ptr);
    }

    #[test]
    fn test_get_string_from_list_out_of_bounds() {
        let strings = vec!["hash1".to_string()];
        let list_ptr = create_string_list(strings);

        let result = get_string_from_list(list_ptr, 10);
        assert!(result.is_null());

        free_rust_string_list(list_ptr);
    }

    #[test]
    fn test_get_string_from_list_null() {
        let result = get_string_from_list(std::ptr::null_mut(), 0);
        assert!(result.is_null());
    }

    #[test]
    fn test_get_resource_info_json_null_inputs() {
        let hash = CString::new("00F1BA373CABEEE1").unwrap();
        assert!(get_resource_info_json(std::ptr::null(), std::ptr::null(), test_log_callback).is_null());
        assert!(get_resource_info_json(hash.as_ptr(), std::ptr::null(), test_log_callback).is_null());
    }

    #[test]
    fn test_free_rust_string_list_null() {
        // Should not panic
        free_rust_string_list(std::ptr::null_mut());
    }

    #[test]
    fn test_free_string_null() {
        // Should not panic
        free_string(std::ptr::null_mut());
    }

    #[test]
    fn test_free_hash_list_null() {
        // Should not panic
        free_hash_list(std::ptr::null_mut());
    }

    #[test]
    fn test_free_hashset_string_null() {
        // Should not panic
        free_hashset_string(std::ptr::null_mut());
    }

    #[test]
    fn test_free_partition_manager_null() {
        // Should not panic
        free_partition_manager(std::ptr::null_mut());
    }

    #[test]
    fn test_free_entities_json_null() {
        // Should not panic
        free_entities_json(std::ptr::null_mut());
    }

    #[test]
    fn test_option_string_to_c_char_some() {
        let test_str = Some("test string".to_string());
        let c_char_ptr = option_string_to_c_char(test_str);

        assert!(!c_char_ptr.is_null());

        unsafe {
            let result = CStr::from_ptr(c_char_ptr).to_str().unwrap();
            assert_eq!(result, "test string");
            let _ = CString::from_raw(c_char_ptr);
        }
    }

    #[test]
    fn test_option_string_to_c_char_none() {
        let c_char_ptr = option_string_to_c_char(None);
        assert!(c_char_ptr.is_null());
    }

    #[test]
    fn test_hash_list_get_version_null() {
        let version = hash_list_get_version(std::ptr::null());
        assert_eq!(version, 0);
    }

    #[test]
    fn test_hash_list_get_all_hashes_null() {
        let list_ptr = hash_list_get_all_hashes(std::ptr::null());
        assert!(!list_ptr.is_null());

        unsafe {
            let list = &*list_ptr;
            assert_eq!(list.length, 0);
        }

        free_rust_string_list(list_ptr);
    }

    #[test]
    fn test_hash_list_get_path_by_hash_null_list() {
        let hash = CString::new("test_hash").unwrap();
        let result = hash_list_get_path_by_hash(std::ptr::null(), hash.as_ptr());
        assert!(result.is_null());
    }

    #[test]
    fn test_hash_list_get_path_by_hash_null_hash() {
        // Create a minimal test - we can't easily construct HashList, so we test with null list too
        // The function should handle null hash gracefully even with a valid list
        let result = hash_list_get_path_by_hash(std::ptr::null(), std::ptr::null());
        assert!(result.is_null());
    }

    #[test]
    fn test_hash_list_get_hint_by_hash_null_list() {
        let hash = CString::new("test_hash").unwrap();
        let result = hash_list_get_hint_by_hash(std::ptr::null(), hash.as_ptr());
        assert!(result.is_null());
    }

    #[test]
    fn test_hash_list_get_resource_type_by_hash_null_list() {
        let hash = CString::new("test_hash").unwrap();
        let result = hash_list_get_resource_type_by_hash(std::ptr::null(), hash.as_ptr());
        assert_eq!(result, 0);
    }

    #[test]
    fn test_hash_list_get_resource_type_by_hash_null_hash() {
        // Create a minimal test - we can't easily construct HashList, so we test with null list too
        // The function should handle null hash gracefully even with a valid list
        let result = hash_list_get_resource_type_by_hash(std::ptr::null(), std::ptr::null());
        assert_eq!(result, 0);
    }
}
