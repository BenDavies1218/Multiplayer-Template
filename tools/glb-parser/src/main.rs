//! GLB Parser — extracts node names from a GLB file and outputs a
//! dynamic_objects_config.json template.
//!
//! Parses the GLB binary format directly to extract the JSON chunk,
//! avoiding the `gltf` crate's strict extension validation.
//! Detects node types (mesh, light, camera, empty) and extracts
//! light metadata from the KHR_lights_punctual extension.

use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "glb-parser",
    about = "Extract node names from GLB and generate dynamic objects config"
)]
struct Cli {
    /// Path to the input GLB file
    input: PathBuf,

    /// Output JSON path (default: stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,
}

// --- Output structs ---

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LightInfo {
    light_type: String,
    color: [f32; 3],
    intensity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct NodeConfig {
    #[serde(rename = "type", default)]
    node_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    light_info: Option<LightInfo>,
    #[serde(default)]
    triggers: Vec<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    state: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct DynamicObjectsConfig {
    nodes: HashMap<String, NodeConfig>,
}

// --- glTF input structs ---

#[derive(Deserialize)]
struct GltfRoot {
    #[serde(default)]
    nodes: Vec<GltfNode>,
    #[serde(default)]
    extensions: Option<GltfRootExtensions>,
}

#[derive(Deserialize)]
struct GltfNode {
    name: Option<String>,
    mesh: Option<usize>,
    camera: Option<usize>,
    #[serde(default)]
    extensions: Option<GltfNodeExtensions>,
}

#[derive(Deserialize)]
struct GltfNodeExtensions {
    #[serde(rename = "KHR_lights_punctual")]
    khr_lights_punctual: Option<KhrLightRef>,
}

#[derive(Deserialize)]
struct KhrLightRef {
    light: usize,
}

#[derive(Deserialize)]
struct GltfRootExtensions {
    #[serde(rename = "KHR_lights_punctual")]
    khr_lights_punctual: Option<KhrLightsPunctual>,
}

#[derive(Deserialize)]
struct KhrLightsPunctual {
    lights: Vec<GltfLight>,
}

#[derive(Deserialize)]
struct GltfLight {
    #[allow(dead_code)]
    name: Option<String>,
    #[serde(rename = "type")]
    light_type: Option<String>,
    #[serde(default = "default_light_color")]
    color: [f32; 3],
    #[serde(default = "default_light_intensity")]
    intensity: f32,
}

fn default_light_color() -> [f32; 3] {
    [1.0, 1.0, 1.0]
}

fn default_light_intensity() -> f32 {
    1.0
}

// --- Detected node info ---

struct DetectedNode {
    name: String,
    node_type: String,
    light_info: Option<LightInfo>,
}

/// Extract the JSON chunk from a GLB binary.
/// GLB format: 12-byte header, then chunks. First chunk (type 0x4E4F534A) is JSON.
fn extract_glb_json(data: &[u8]) -> Result<&[u8], String> {
    if data.len() < 12 {
        return Err("File too small for GLB header".to_string());
    }
    let magic = u32::from_le_bytes(data[0..4].try_into().unwrap());
    if magic != 0x46546C67 {
        return Err("Not a GLB file (bad magic)".to_string());
    }

    // First chunk starts at offset 12
    if data.len() < 20 {
        return Err("File too small for first chunk header".to_string());
    }
    let chunk_length = u32::from_le_bytes(data[12..16].try_into().unwrap()) as usize;
    let chunk_type = u32::from_le_bytes(data[16..20].try_into().unwrap());

    if chunk_type != 0x4E4F534A {
        return Err("First chunk is not JSON".to_string());
    }

    let json_end = 20 + chunk_length;
    if data.len() < json_end {
        return Err("JSON chunk extends beyond file".to_string());
    }

    Ok(&data[20..json_end])
}

fn detect_node_type(node: &GltfNode, root: &GltfRoot) -> DetectedNode {
    let name = node.name.clone().unwrap_or_default();

    // Check for light (KHR_lights_punctual extension on node)
    if let Some(ref exts) = node.extensions
        && let Some(ref light_ref) = exts.khr_lights_punctual
    {
        let light_info = root
            .extensions
            .as_ref()
            .and_then(|re| re.khr_lights_punctual.as_ref())
            .and_then(|lp| lp.lights.get(light_ref.light))
            .map(|light| LightInfo {
                light_type: light
                    .light_type
                    .clone()
                    .unwrap_or_else(|| "point".to_string()),
                color: light.color,
                intensity: light.intensity,
            });

        return DetectedNode {
            name,
            node_type: "light".to_string(),
            light_info,
        };
    }

    // Check for mesh
    if node.mesh.is_some() {
        return DetectedNode {
            name,
            node_type: "mesh".to_string(),
            light_info: None,
        };
    }

    // Check for camera
    if node.camera.is_some() {
        return DetectedNode {
            name,
            node_type: "camera".to_string(),
            light_info: None,
        };
    }

    // Default: empty
    DetectedNode {
        name,
        node_type: "empty".to_string(),
        light_info: None,
    }
}

fn main() {
    let cli = Cli::parse();

    let file_data = std::fs::read(&cli.input).unwrap_or_else(|e| {
        eprintln!("Error reading file '{}': {}", cli.input.display(), e);
        std::process::exit(1);
    });

    let json_bytes = extract_glb_json(&file_data).unwrap_or_else(|e| {
        eprintln!("Error parsing GLB '{}': {}", cli.input.display(), e);
        std::process::exit(1);
    });

    let gltf_root: GltfRoot = serde_json::from_slice(json_bytes).unwrap_or_else(|e| {
        eprintln!("Error parsing glTF JSON: {}", e);
        std::process::exit(1);
    });

    // Detect node types
    let mut detected_nodes: Vec<DetectedNode> = gltf_root
        .nodes
        .iter()
        .filter(|n| n.name.as_ref().is_some_and(|name| !name.is_empty()))
        .map(|n| detect_node_type(n, &gltf_root))
        .collect();

    // Sort by type then name
    detected_nodes.sort_by(|a, b| a.node_type.cmp(&b.node_type).then(a.name.cmp(&b.name)));

    // Load existing config if output file exists (merge mode)
    let mut config = if let Some(ref output_path) = cli.output {
        if output_path.exists() {
            let contents = std::fs::read_to_string(output_path).unwrap_or_else(|e| {
                eprintln!(
                    "Error reading existing config '{}': {}",
                    output_path.display(),
                    e
                );
                std::process::exit(1);
            });
            let existing: DynamicObjectsConfig =
                serde_json::from_str(&contents).unwrap_or_else(|e| {
                    eprintln!("Error parsing existing config: {}", e);
                    std::process::exit(1);
                });
            eprintln!(
                "Merging with existing config ({} nodes)",
                existing.nodes.len()
            );

            // Warn about nodes in config that are missing from GLB
            let node_names: Vec<&str> = detected_nodes.iter().map(|n| n.name.as_str()).collect();
            for key in existing.nodes.keys() {
                if !node_names.contains(&key.as_str()) {
                    eprintln!("WARNING: config node '{}' not found in GLB", key);
                }
            }

            existing
        } else {
            DynamicObjectsConfig::default()
        }
    } else {
        DynamicObjectsConfig::default()
    };

    // Add new nodes or update existing "empty" nodes with detected type info
    let mut added = 0;
    let mut updated = 0;
    for detected in &detected_nodes {
        if let Some(existing) = config.nodes.get_mut(&detected.name) {
            // Update type/light_info if existing node was "empty" and we now have better info
            if existing.node_type == "empty" && detected.node_type != "empty" {
                existing.node_type = detected.node_type.clone();
                existing.light_info = detected.light_info.clone();
                updated += 1;
            }
        } else {
            config.nodes.insert(
                detected.name.clone(),
                NodeConfig {
                    node_type: detected.node_type.clone(),
                    light_info: detected.light_info.clone(),
                    triggers: Vec::new(),
                    state: None,
                },
            );
            added += 1;
        }
    }

    // Type summary
    let mut type_counts: HashMap<&str, usize> = HashMap::new();
    for detected in &detected_nodes {
        *type_counts.entry(detected.node_type.as_str()).or_default() += 1;
    }
    eprintln!(
        "GLB nodes: {}, new: {}, updated: {}, total config nodes: {}",
        detected_nodes.len(),
        added,
        updated,
        config.nodes.len()
    );
    let mut type_summary: Vec<_> = type_counts.iter().collect();
    type_summary.sort_by_key(|(t, _)| *t);
    for (node_type, count) in &type_summary {
        eprintln!("  {}: {}", node_type, count);
    }

    // Output
    let json = serde_json::to_string_pretty(&config).expect("Failed to serialize config");

    if let Some(ref output_path) = cli.output {
        std::fs::write(output_path, &json).unwrap_or_else(|e| {
            eprintln!("Error writing '{}': {}", output_path.display(), e);
            std::process::exit(1);
        });
        eprintln!("Written to {}", output_path.display());
    } else {
        println!("{}", json);
    }
}
