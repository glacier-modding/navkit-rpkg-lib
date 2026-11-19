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

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    extern "C" fn test_log_callback(_msg: *const std::os::raw::c_char) {}

    #[test]
    fn test_deserialize_complete_entities_json() {
        let json_str = r#"{
            "meshes": [
                {
                    "alocHash": "00123ABC",
                    "primHash": "00456DEF",
                    "entity": {
                        "id": "entity1",
                        "name": "TestEntity",
                        "tblu": "template1",
                        "position": {"x": 1.0, "y": 2.0, "z": 3.0},
                        "rotation": {"x": 0.0, "y": 0.0, "z": 0.0, "w": 1.0},
                        "scale": {"type": "uniform", "data": {"x": 1.0, "y": 1.0, "z": 1.0}}
                    }
                }
            ],
            "pfBoxes": [
                {
                    "id": "box1",
                    "position": {"x": 0.0, "y": 0.0, "z": 0.0},
                    "rotation": {"x": 0.0, "y": 0.0, "z": 0.0, "w": 1.0},
                    "type": {"type": "box", "data": "somedata"},
                    "scale": {"type": "uniform", "data": {"x": 1.0, "y": 1.0, "z": 1.0}}
                }
            ],
            "pfSeedPoints": [
                {
                    "id": "seed1",
                    "position": {"x": 5.0, "y": 6.0, "z": 7.0},
                    "rotation": {"x": 0.0, "y": 0.0, "z": 0.0, "w": 1.0}
                }
            ]
        }"#;

        let result: Result<EntitiesJson, _> = serde_json::from_str(json_str);
        assert!(result.is_ok());

        let entities = result.unwrap();
        assert_eq!(entities.meshes.len(), 1);
        assert_eq!(entities.pf_boxes.len(), 1);
        assert_eq!(entities.pf_seed_points.len(), 1);

        assert_eq!(entities.meshes[0].aloc_hash, "00123ABC");
        assert_eq!(entities.meshes[0].prim_hash, "00456DEF");
        assert_eq!(entities.meshes[0].entity.id, "entity1");
    }

    #[test]
    fn test_build_from_nav_json_string_valid() {
        let json_str = r#"{
            "meshes": [],
            "pfBoxes": [],
            "pfSeedPoints": []
        }"#.to_string();

        let result = EntitiesJson::build_from_nav_json_string(json_str, test_log_callback);
        assert!(result.is_some());

        let entities = result.unwrap();
        assert_eq!(entities.meshes.len(), 0);
        assert_eq!(entities.pf_boxes.len(), 0);
        assert_eq!(entities.pf_seed_points.len(), 0);
    }

    #[test]
    fn test_build_from_nav_json_string_invalid() {
        let json_str = "invalid json".to_string();

        let result = EntitiesJson::build_from_nav_json_string(json_str, test_log_callback);
        assert!(result.is_none());
    }

    #[test]
    fn test_build_from_nav_json_file_nonexistent() {
        let result = EntitiesJson::build_from_nav_json_file(
            "/nonexistent/file.json".to_string(),
            test_log_callback,
        );
        assert!(result.is_none());
    }

    #[test]
    fn test_build_from_nav_json_file_valid() {
        let json_content = r#"{
            "meshes": [
                {
                    "alocHash": "hash1",
                    "primHash": "prim1",
                    "entity": {
                        "id": "1",
                        "position": {"x": 0.0, "y": 0.0, "z": 0.0},
                        "rotation": {"x": 0.0, "y": 0.0, "z": 0.0, "w": 1.0},
                        "scale": {"type": "uniform", "data": {"x": 1.0, "y": 1.0, "z": 1.0}}
                    }
                }
            ],
            "pfBoxes": [],
            "pfSeedPoints": []
        }"#;

        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_nav.json");
        fs::write(&test_file, json_content).unwrap();

        let result = EntitiesJson::build_from_nav_json_file(
            test_file.to_str().unwrap().to_string(),
            test_log_callback,
        );

        assert!(result.is_some());
        let entities = result.unwrap();
        assert_eq!(entities.meshes.len(), 1);

        // Cleanup
        fs::remove_file(test_file).ok();
    }

    #[test]
    fn test_mesh_hashes_and_entity_fields() {
        let json_str = r#"{
            "alocHash": "ALOC123",
            "primHash": "PRIM456",
            "entity": {
                "id": "entity_id",
                "name": "entity_name",
                "tblu": "tblu_id",
                "position": {"x": 10.0, "y": 20.0, "z": 30.0},
                "rotation": {"x": 0.1, "y": 0.2, "z": 0.3, "w": 0.9},
                "scale": {"type": "nonuniform", "data": {"x": 2.0, "y": 3.0, "z": 4.0}}
            }
        }"#;

        let mesh: MeshHashesAndEntity = serde_json::from_str(json_str).unwrap();
        assert_eq!(mesh.aloc_hash, "ALOC123");
        assert_eq!(mesh.prim_hash, "PRIM456");
        assert_eq!(mesh.entity.id, "entity_id");
        assert_eq!(mesh.entity.name, Some("entity_name".to_string()));
        assert_eq!(mesh.entity.tblu, Some("tblu_id".to_string()));
        assert_eq!(mesh.entity.position.x, 10.0);
        assert_eq!(mesh.entity.rotation.w, 0.9);
        assert_eq!(mesh.entity.scale.r#type, "nonuniform");
    }

    #[test]
    fn test_pf_box_deserialization() {
        let json_str = r#"{
            "id": "box_id",
            "position": {"x": 1.0, "y": 2.0, "z": 3.0},
            "rotation": {"x": 0.0, "y": 0.0, "z": 0.0, "w": 1.0},
            "type": {"type": "blocking", "data": "test_data"},
            "scale": {"type": "uniform", "data": {"x": 1.5, "y": 1.5, "z": 1.5}}
        }"#;

        let pf_box: PfBox = serde_json::from_str(json_str).unwrap();
        assert_eq!(pf_box.id, "box_id");
        assert_eq!(pf_box.position.y, 2.0);
        assert_eq!(pf_box.r#type.r#type, "blocking");
        assert_eq!(pf_box.r#type.data, "test_data");
    }

    #[test]
    fn test_pf_seed_point_deserialization() {
        let json_str = r#"{
            "id": "seed_id",
            "position": {"x": 7.0, "y": 8.0, "z": 9.0},
            "rotation": {"x": 0.5, "y": 0.5, "z": 0.5, "w": 0.5}
        }"#;

        let seed_point: PfSeedPoint = serde_json::from_str(json_str).unwrap();
        assert_eq!(seed_point.id, "seed_id");
        assert_eq!(seed_point.position.x, 7.0);
        assert_eq!(seed_point.rotation.x, 0.5);
    }

    #[test]
    fn test_vec3_deserialization() {
        let json_str = r#"{"x": 1.5, "y": 2.5, "z": 3.5}"#;
        let vec3: Vec3 = serde_json::from_str(json_str).unwrap();
        assert_eq!(vec3.x, 1.5);
        assert_eq!(vec3.y, 2.5);
        assert_eq!(vec3.z, 3.5);
    }

    #[test]
    fn test_rotation_deserialization() {
        let json_str = r#"{"x": 0.1, "y": 0.2, "z": 0.3, "w": 0.4}"#;
        let rotation: Rotation = serde_json::from_str(json_str).unwrap();
        assert_eq!(rotation.x, 0.1);
        assert_eq!(rotation.y, 0.2);
        assert_eq!(rotation.z, 0.3);
        assert_eq!(rotation.w, 0.4);
    }

    #[test]
    fn test_scale_deserialization() {
        let json_str = r#"{"type": "uniform", "data": {"x": 2.0, "y": 2.0, "z": 2.0}}"#;
        let scale: Scale = serde_json::from_str(json_str).unwrap();
        assert_eq!(scale.r#type, "uniform");
        assert_eq!(scale.data.x, 2.0);
    }

    #[test]
    fn test_entity_with_optional_fields_none() {
        let json_str = r#"{
            "id": "test_id",
            "position": {"x": 0.0, "y": 0.0, "z": 0.0},
            "rotation": {"x": 0.0, "y": 0.0, "z": 0.0, "w": 1.0},
            "scale": {"type": "uniform", "data": {"x": 1.0, "y": 1.0, "z": 1.0}}
        }"#;

        let aloc: Aloc = serde_json::from_str(json_str).unwrap();
        assert_eq!(aloc.id, "test_id");
        assert!(aloc.name.is_none());
        assert!(aloc.tblu.is_none());
    }
}
