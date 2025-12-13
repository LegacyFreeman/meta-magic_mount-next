use std::{collections::HashMap, io::Cursor, path::Path};

use anyhow::{Context, Result};
use java_properties::PropertiesIter;
use serde::Serialize;

use crate::defs::{DISABLE_FILE_NAME, REMOVE_FILE_NAME, SKIP_MOUNT_FILE_NAME};

#[derive(Debug, Serialize)]
pub struct ModuleInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub disabled: bool,
    pub skip: bool,
}

fn read_prop<P>(path: P) -> Result<HashMap<String, String>>
where
    P: AsRef<Path>,
{
    let prop_path = path.as_ref().join("module.prop");
    let content = std::fs::read_to_string(&prop_path)
        .with_context(|| format!("Failed to read module.prop: {}", prop_path.display()))?;

    let mut prop_map: HashMap<String, String> = HashMap::new();
    PropertiesIter::new_with_encoding(Cursor::new(content), encoding_rs::UTF_8)
        .read_into(|k, v| {
            prop_map.insert(k, v);
        })
        .with_context(|| format!("Failed to parse module.prop: {}", prop_path.display()))?;

    Ok(prop_map)
}

/// Scans for modules that will be actually mounted by `magic_mount`.
/// Filters out modules that:
/// 1. Do not have a `system` directory.
/// 2. Are disabled or removed.
/// 3. Have the `skip_mount` flag.
pub fn scan_modules<P>(module_dir: P) -> Vec<ModuleInfo>
where
    P: AsRef<Path>,
{
    let mut modules = Vec::new();

    if let Ok(entries) = module_dir.as_ref().read_dir() {
        for entry in entries.flatten() {
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            if !path.join("module.prop").exists() {
                continue;
            }

            if !path.join("system").is_dir() {
                continue;
            }

            let disabled =
                path.join(DISABLE_FILE_NAME).exists() || path.join(REMOVE_FILE_NAME).exists();
            let skip = path.join(SKIP_MOUNT_FILE_NAME).exists();
            if disabled || skip {
                continue;
            }

            let id = entry.file_name().to_string_lossy().to_string();

            let Ok(prop_map) = read_prop(path) else {
                continue;
            };

            let name = prop_map
                .get("name")
                .map_or_else(|| "unknown".to_string(), std::clone::Clone::clone);
            let version = prop_map
                .get("version")
                .map_or_else(|| "unknown".to_string(), std::clone::Clone::clone);
            let author = prop_map
                .get("author")
                .map_or_else(|| "unknown".to_string(), std::clone::Clone::clone);
            let description = prop_map
                .get("description")
                .map_or_else(|| "unknown".to_string(), std::clone::Clone::clone);

            modules.push(ModuleInfo {
                id,
                name,
                version,
                author,
                description,
                disabled,
                skip,
            });
        }
    }
    modules.sort_by(|a, b| a.id.cmp(&b.id));

    modules
}
