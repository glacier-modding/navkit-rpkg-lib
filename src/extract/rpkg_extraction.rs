use crate::{json_serde::entities_json::EntitiesJson, package::package_scan::PackageScan};
use anyhow::{bail, Context};
use hitman_commons::hash_list::HashList;
use hitman_commons::metadata::ExtendedResourceMetadata;
use hitman_formats::material::MaterialInstance;
use rpkg_rs::resource::{
    partition_manager::PartitionManager, resource_package::ResourcePackage,
    runtime_resource_id::RuntimeResourceID,
};
use serde::Serialize;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::slice;
use std::thread::{self};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

pub struct RpkgExtraction;

impl RpkgExtraction {
    pub const HASH_LIST_VERSION_ENDPOINT: &str =
        "https://github.com/glacier-modding/Hitman-Hashes/releases/latest/download/version";

    pub const HASH_LIST_ENDPOINT: &str =
        "https://github.com/glacier-modding/Hitman-Hashes/releases/latest/download/hash_list.sml";

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
                        let resource_contents = match rpkg.read_resource(&rrid) {
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

    pub fn get_hash_list_from_file_or_repo(output_folder: String,
                                           log_callback: extern "C" fn(*const c_char),) -> anyhow::Result<HashList> {
        let msg = std::ffi::CString::new("Loading hash list from file.".to_string())?;
        log_callback(msg.as_ptr());
        let hash_list_path = PathBuf::from(&output_folder).join("hash_list.sml");
        let hash_list = match fs::read(hash_list_path.clone()) {
            Ok(bytes) => serde_smile::from_slice(&bytes).ok(),
            Err(_e) => {
                let msg = std::ffi::CString::new("No hash list file.".to_string())?;
                log_callback(msg.as_ptr());
                None
            },
        };
        
        if let Ok(response) = reqwest::blocking::get(RpkgExtraction::HASH_LIST_VERSION_ENDPOINT) {
            if let Ok(data) = response.text() {
                let msg = std::ffi::CString::new("Checking latest hash list version.".to_string())?;
                log_callback(msg.as_ptr());
                let new_version = data
                    .trim()
                    .parse::<u32>()
                    .context("Online hash list version wasn't a number")?;
                
                let current_version = hash_list.as_ref().map(|h: &HashList| h.version).unwrap_or(0);

                if current_version >= new_version {
                    let msg = std::ffi::CString::new("Already have latest hash list.".to_string())?;
                    log_callback(msg.as_ptr());
                    return hash_list.ok_or_else(|| anyhow::anyhow!("Local hash list missing."));
                }
                
                if let Ok(response) = reqwest::blocking::get(RpkgExtraction::HASH_LIST_ENDPOINT) {
                    if let Ok(data) = response.bytes() {
                        let msg = std::ffi::CString::new("Newer hash list version found. Downloading.".to_string())?;
                        log_callback(msg.as_ptr());
                        let new_hash_list = HashList::from_compressed(&data)?;

                        fs::write(hash_list_path, serde_smile::to_vec(&new_hash_list)?)?;
                        let msg = std::ffi::CString::new("Finished downloading latest hash list.".to_string())?;
                        log_callback(msg.as_ptr());
                        return Ok(new_hash_list);
                    }
                }
            }
        }
        
        hash_list.ok_or_else(|| anyhow::anyhow!("Failed to retrieve hash list."))
    }

    // From GlacierKit
    pub fn extract_latest_resource(
        resource_id: RuntimeResourceID,
        partition_manager: &PartitionManager,
    ) -> anyhow::Result<(ExtendedResourceMetadata, Vec<u8>)> {
        for partition in &partition_manager.partitions {
            if let Some((info, _)) = partition
                .latest_resources()
                .into_iter()
                .find(|(x, _)| *x.rrid() == resource_id)
            {
                return Ok((
                    info.try_into()
                        .with_context(|| format!("Couldn't extract resource {resource_id}"))?,
                    partition
                        .read_resource(&resource_id)
                        .with_context(|| format!("Couldn't extract {resource_id} using rpkg-rs"))?,
                ));
            }
        }

        bail!("Couldn't find {resource_id} in any partition when extracting resource");
    }

    pub fn get_mati_json_by_hash(
        resource_hash: String,
        hash_list: &HashList,
        partition_manager: &PartitionManager,
        log_callback: extern "C" fn(*const c_char),
    ) -> anyhow::Result<String> {
        let msg = std::ffi::CString::new(
            format!("Getting Mati json for {} in Rpkg files.", resource_hash).to_string(),
        )?;
        log_callback(msg.as_ptr());
        let (res_meta, res_data) = RpkgExtraction::extract_latest_resource(
            RuntimeResourceID::from_hex_string(resource_hash.as_str()).map_err(|_| anyhow::anyhow!("Invalid resource hash"))?,
            partition_manager,
        )?;

        let material = MaterialInstance::parse(
            &res_data,
            &res_meta.core_info.with_hash_list(&hash_list.entries),
        )
        .context("Couldn't parse material instance")?;

        let mut buf = Vec::new();
        let formatter = serde_json::ser::PrettyFormatter::with_indent(b"\t");
        let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);

        material.serialize(&mut ser)?;

        Ok(String::from_utf8(buf)?)
    }

    pub fn get_all_referenced_hashes_by_hash_from_rpkg_files(
        resource_hash: String,
        partition_manager: &PartitionManager,
        log_callback: extern "C" fn(*const c_char),
    ) -> anyhow::Result<Vec<String>> {
        let msg = std::ffi::CString::new(
            format!("Getting references for {} in Rpkg files.", resource_hash).to_string(),
        )?;
        log_callback(msg.as_ptr());
        let msg = std::ffi::CString::new(
            format!("Converting rrid for {} in Rpkg files.", resource_hash).to_string(),
        )?;
        log_callback(msg.as_ptr());
        let rrid = RuntimeResourceID::from_hex_string(resource_hash.as_str()).map_err(|_| anyhow::anyhow!("Invalid resource hash"))?;
        let msg = std::ffi::CString::new(
            format!("Getting partitions for {} in Rpkg files.", resource_hash).to_string(),
        )?;
        log_callback(msg.as_ptr());
        for partition in &partition_manager.partitions {
            let msg = std::ffi::CString::new(
                format!("Checking partition {} for {}.", partition.partition_info().id, resource_hash).to_string(),
            )?;
            log_callback(msg.as_ptr());
            if let Some((info, _)) = partition
                .latest_resources()
                .into_iter()
                .find(|(x, _)| *x.rrid() == rrid)
            {
                let msg = std::ffi::CString::new(
                    format!("Found hash {} in partition {}.", resource_hash, partition.partition_info().id).to_string(),
                )?;
                log_callback(msg.as_ptr());

                let result = info
                    .references()
                    .iter()
                    .map(|(rrid, _)| rrid.to_hex_string())
                    .collect();
                let msg = std::ffi::CString::new(
                    format!("Got references for {} in Rpkg files.", resource_hash).to_string(),
                )?;
                log_callback(msg.as_ptr());

                return Ok(result);
            } else {
                let msg = std::ffi::CString::new(
                    format!("Didn't find hash {} in partition {}.", resource_hash, partition.partition_info().id).to_string(),
                )?;
                log_callback(msg.as_ptr());

            }
        }
        bail!("Couldn't find {rrid} in any partition when extracting referenced resource");
    }

    pub fn get_all_resources_hashes_by_type_from_rpkg_files(
        partition_manager: &PartitionManager,
        resource_type: String,
        log_callback: extern "C" fn(*const c_char),
    ) -> Vec<String> {
        let resources: Vec<_> = partition_manager
            .partitions
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
        resources.iter().for_each(|resource| {
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
