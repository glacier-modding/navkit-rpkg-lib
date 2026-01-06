use serde::{Deserialize, Serialize};
use std::fs;
use std::os::raw::c_char;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntitiesJson {
    pub meshes: Vec<MeshHashesAndEntity>,
    #[serde(rename = "pfBoxes")]
    pub pf_boxes: Vec<PfBox>,
    #[serde(rename = "pfSeedPoints")]
    pub pf_seed_points: Vec<PfSeedPoint>,
}

impl EntitiesJson {
    pub fn build_from_nav_json_file(
        nav_json_file: String,
        log_callback: extern "C" fn(*const c_char),
    ) -> Option<EntitiesJson> {
        let msg = std::ffi::CString::new(format!(
            "Loading scene from nav.json file: {}",
            nav_json_file
        ))
        .unwrap();
        log_callback(msg.as_ptr());

        let nav_json_string = match fs::read_to_string(nav_json_file.as_str()) {
            Ok(c) => c,
            Err(e) => {
                let msg =
                    std::ffi::CString::new(format!("Error reading nav.json file: {}", e)).unwrap();
                log_callback(msg.as_ptr());
                return None;
            }
        };
        match EntitiesJson::build_from_nav_json_string(nav_json_string, log_callback) {
            Some(json) => Some(json),
            None => None,
        }
    }

    pub fn build_from_nav_json_string(
        nav_json_string: String,
        log_callback: extern "C" fn(*const c_char),
    ) -> Option<EntitiesJson> {
        match serde_json::from_str(&nav_json_string) {
            Ok(json) => Some(json),
            Err(e) => {
                let msg =
                    std::ffi::CString::new(format!("Error parsing nav.json file: {}", e)).unwrap();
                log_callback(msg.as_ptr());
                None
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrickMessage {
    pub brick_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeshHashesAndEntity {
    pub aloc_hash: String,
    pub prim_hash: String,
    pub entity: Aloc,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Aloc {
    pub id: String,
    pub name: Option<String>,
    pub tblu: Option<String>,
    pub position: Vec3,
    pub rotation: Rotation,
    pub scale: Scale,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PfBox {
    pub id: String,
    pub position: Vec3,
    pub rotation: Rotation,
    #[serde(rename = "type")]
    pub r#type: Type,
    pub scale: Scale,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PfSeedPoint {
    pub id: String,
    pub position: Vec3,
    pub rotation: Rotation,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde()]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rotation {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scale {
    #[serde(rename = "type")]
    pub r#type: String,
    pub data: Vec3,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Type {
    #[serde(rename = "type")]
    pub r#type: String,
    pub data: String,
}
