//! GLB Parser — extracts node names from a GLB file and outputs a
//! dynamic_objects_config.json template.
//!
//! Parses the GLB binary format directly to extract the JSON chunk,
//! avoiding the `gltf` crate's strict extension validation.

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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct NodeConfig {
    #[serde(default)]
    triggers: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct DynamicObjectsConfig {
    nodes: HashMap<String, NodeConfig>,
}

/// Minimal glTF JSON root — we only care about nodes.
#[derive(Deserialize)]
struct GltfRoot {
    #[serde(default)]
    nodes: Vec<GltfNode>,
}

#[derive(Deserialize)]
struct GltfNode {
    name: Option<String>,
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

    // Collect all named nodes
    let mut node_names: Vec<String> = gltf_root
        .nodes
        .iter()
        .filter_map(|n| n.name.as_ref())
        .filter(|n| !n.is_empty())
        .cloned()
        .collect();
    node_names.sort();

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
            for key in existing.nodes.keys() {
                if !node_names.contains(key) {
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

    // Add new nodes (don't overwrite existing)
    let mut added = 0;
    for name in &node_names {
        if !config.nodes.contains_key(name) {
            config.nodes.insert(name.clone(), NodeConfig::default());
            added += 1;
        }
    }

    eprintln!(
        "GLB nodes: {}, new: {}, total config nodes: {}",
        node_names.len(),
        added,
        config.nodes.len()
    );

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
