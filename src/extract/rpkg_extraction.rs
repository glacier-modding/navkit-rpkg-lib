use rpkg_rs::resource::{
    partition_manager::PartitionManager, resource_package::ResourcePackage,
    runtime_resource_id::RuntimeResourceID,
};
use std::ffi::CStr;
use std::os::raw::c_char;
use std::slice;
use std::thread::{self};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use crate::{json_serde::entities_json::EntitiesJson, package::package_scan::PackageScan};

pub struct RpkgExtraction;

impl RpkgExtraction {
    pub unsafe fn extract_resources_from_rpkg(
        runtime_folder: String,
        needed_hashes: *const *const c_char,
        needed_hashes_len: usize,
        partition_manager: &PartitionManager,
        output_folder: String,
        resource_type: String,
        log_callback: extern "C" fn(*const c_char),
    ) -> std::ffi::c_int {
        let mut needed_hashes_list = Vec::new();
        if !needed_hashes.is_null() {
            let slice = slice::from_raw_parts(needed_hashes, needed_hashes_len);
            for &ptr in slice {
                if !ptr.is_null() {
                    if let Ok(s) = CStr::from_ptr(ptr).to_str() {
                        needed_hashes_list.push(s.to_string());
                    }
                }
            }
        }
        let resource_count = needed_hashes_list.len();
        let target_num_threads = 10;
        let output_folder_ref = &output_folder;
        let runtime_folder_ref = &runtime_folder;
        let resource_type_ref = &resource_type;
        let msg = std::ffi::CString::new(format!(
            "Creating directory '{}' if it doesn't exist.",
            output_folder
        ))
        .unwrap();
        log_callback(msg.as_ptr());
        if let Err(e) = fs::create_dir_all(output_folder_ref) {
            let msg =
                std::ffi::CString::new(format!("Failed to create resource folder: {}", e)).unwrap();
            log_callback(msg.as_ptr());
            return -1;
        }
        let msg = std::ffi::CString::new(format!(
            "Extracting using {} threads of around {} resources per thread...",
            target_num_threads,
            resource_count / target_num_threads
        ))
        .unwrap();
        log_callback(msg.as_ptr());
        let alocs_or_prims_output_folder_path = PathBuf::from(&output_folder);
        let alocs_or_prims_output_folder_path_ref = &alocs_or_prims_output_folder_path;
        let success = thread::scope(|scope| {
            let mut handles = Vec::new();
            for (chunk_i, chunk) in needed_hashes_list
                .chunks(resource_count.div_ceil(target_num_threads))
                .enumerate()
            {
                handles.push(scope.spawn(move || {
                    let output_folder_path =
                        PathBuf::from(String::from(output_folder_ref));
                    let mut resource_packages: HashMap<String, ResourcePackage> = HashMap::new();

                    let mut skipped = 0;
                    let mut extracted = 0;
                    for hash in chunk {
                        let runtime_folder_path = PathBuf::from(runtime_folder_ref);

                        let rrid: RuntimeResourceID =
                            match RuntimeResourceID::from_hex_string(hash.as_str()) {
                                Ok(id) => id,
                                Err(_) => {
                                    let msg = std::ffi::CString::new(format!(
                                        "Error getting RRID from hash: {}",
                                        hash.as_str()
                                    ))
                                    .unwrap();
                                    log_callback(msg.as_ptr());
                                    return Err(());
                                }
                            };
                        let resource_info = match
                            PackageScan::get_resource_info(partition_manager, &rrid) {
                                Some(info) => info,
                                None => {
                                    let msg = std::ffi::CString::new(format!(
                                        "Error getting resource info for hash: {}",
                                        hash.as_str()
                                    ))
                                    .unwrap();
                                    log_callback(msg.as_ptr());
                                    return Err(());
                                }
                        };
                        let last_partition = resource_info.last_partition;
                        let package_path_buf = runtime_folder_path.join(last_partition.clone());
                        let package_path = Path::new(&package_path_buf);
                        let aloc_or_prim_file_path_buf =
                            alocs_or_prims_output_folder_path_ref.join(hash.clone() + "." + &resource_type_ref);
                        let aloc_or_prim_file_path_buf = aloc_or_prim_file_path_buf.as_os_str().to_str().unwrap();
                        let aloc_or_prim_file_path = Path::new(aloc_or_prim_file_path_buf);
                        if aloc_or_prim_file_path.exists() {
                            let aloc_or_prim_file_path_metadata = aloc_or_prim_file_path.metadata();
                            if aloc_or_prim_file_path_metadata.unwrap().modified().unwrap() >= package_path.metadata().unwrap().modified().unwrap() {
                                skipped += 1;
                                continue
                            }
                        }
                        let rpkg = match resource_packages.entry(last_partition.clone()) {
                            std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
                            std::collections::hash_map::Entry::Vacant(entry) => {
                                match ResourcePackage::from_file(&package_path_buf) {
                                    Ok(pkg) => entry.insert(pkg),
                                    Err(e) => {
                                        let msg = std::ffi::CString::new(format!(
                                            "Failed parse resource package: {}",
                                            e
                                        ))
                                        .unwrap();
                                        log_callback(msg.as_ptr());
                                        return Err(());
                                    }
                                }
                            }
                        };
                        let resource_contents = match rpkg.read_resource(&package_path_buf, &rrid) {
                            Ok(c) => c,
                            Err(e) => {
                                let msg = std::ffi::CString::new(format!(
                                    "Failed extract resource: {}",
                                    e
                                ))
                                .unwrap();
                                log_callback(msg.as_ptr());
                                return Err(());
                            }
                        };

                        let file_extension: String;
                        if resource_type_ref == "ALOC" {
                            file_extension = ".ALOC".to_string();
                        } else if resource_type_ref == "PRIM" {
                            file_extension = ".PRIM".to_string();
                        } else if resource_type_ref == "NAVP" {
                            file_extension = ".NAVP".to_string();
                        } else {
                            file_extension = ".AIRG".to_string();
                        }
                        let resource_file_path_buf =
                            output_folder_path.join(hash.clone() + &file_extension);
                        let resource_file_path =
                            resource_file_path_buf.as_os_str().to_str().unwrap();
                        extracted += 1;

                        if let Err(e) = fs::write(resource_file_path, resource_contents) {
                            let msg =
                                std::ffi::CString::new(format!("File failed to be written: {}", e))
                                    .unwrap();
                            log_callback(msg.as_ptr());
                            return Err(());
                        }
                    }
                    let msg = std::ffi::CString::new(format!(
                        "Thread {}: Extracted {} resources. Skipped extraction of {} resources that are newer than their rpkg file.",
                        chunk_i, extracted, skipped)
                    ).unwrap();
                    log_callback(msg.as_ptr());

                    Ok(())
                }));
            }

            let mut success = true;
            for thread_handle in handles {
                if let Ok(res) = thread_handle.join() {
                    if res.is_err() {
                        success = false;
                    }
                } else {
                    success = false;
                }
            }
            success
        });

        if success {
            0
        } else {
            -1
        }
    }

    pub fn get_needed_aloc_or_prim_hashes_from_scene(
        scene_nav_json: &EntitiesJson,
        aloc_or_prim_type: String,
    ) -> HashSet<String> {
        let mut aloc_or_prim_hashes: HashSet<String> = HashSet::new();
        let mut needed_hashes: HashSet<String> = HashSet::new();

        let file_extension: String;
        if aloc_or_prim_type == "ALOC" {
            file_extension = ".ALOC".to_string();
        } else {
            file_extension = ".PRIM".to_string();
        }
        for entity in &scene_nav_json.meshes {
            if aloc_or_prim_type == "ALOC" {
                aloc_or_prim_hashes.insert(entity.aloc_hash.clone());
            } else {
                aloc_or_prim_hashes.insert(entity.prim_hash.clone());
            }
        }
        for hash in aloc_or_prim_hashes {
            needed_hashes.insert(hash);
        }
        needed_hashes
    }

    pub fn get_all_resources_hashes_by_type_from_rpkg_files(
        partition_manager: &PartitionManager,
        resource_type: String,
        log_callback: extern "C" fn(*const c_char),
    ) -> Vec<String> {
        let navps: Vec<_> = partition_manager
            .partitions()
            .iter()
            .flat_map(|partition| {
                partition
                    .latest_resources()
                    .into_iter()
                    .filter_map(|(resource, _)| {
                        (resource.data_type() == resource_type).then_some(resource)
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        let mut resource_hashes: HashSet<String> = HashSet::new();
        navps.iter().for_each(|resource| {
            let rid = resource.rrid();
            resource_hashes.insert(rid.to_hex_string());
        });

        let msg = std::ffi::CString::new(
            format!(
                "Found {} {} Resources in Rpkg files.",
                resource_hashes.len(),
                resource_type
            )
            .to_string(),
        )
        .unwrap();
        log_callback(msg.as_ptr());
        resource_hashes.into_iter().collect()
    }
}
