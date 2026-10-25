// src-tauri/src/form_registry.rs
// Discovers, parses, and hot-reloads .rform.json statistical dialog schemas.
// The Analyze menu follows the directory hierarchy of the modules/ folder.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuNode {
    pub label: String,
    #[serde(default)]
    pub children: Vec<MenuNode>,
    #[serde(default)]
    pub form_id: Option<String>,
    #[serde(default)]
    pub is_custom: bool,
}

pub struct FormRegistry {
    catalog_dir: Arc<RwLock<PathBuf>>,
    user_forms_dir: PathBuf,
    forms: Arc<RwLock<HashMap<String, Value>>>,
    menu_tree: Arc<RwLock<Vec<MenuNode>>>,
}

impl FormRegistry {
    pub fn new() -> Self {
        let catalog_dir = find_catalog_dir();
        let user_forms_dir = get_user_forms_dir();
        Self {
            catalog_dir: Arc::new(RwLock::new(catalog_dir)),
            user_forms_dir,
            forms: Arc::new(RwLock::new(HashMap::new())),
            menu_tree: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn set_catalog_dir(&self, dir: PathBuf) {
        *self.catalog_dir.write().await = dir;
    }

    pub async fn get_catalog_dir(&self) -> PathBuf {
        self.catalog_dir.read().await.clone()
    }

    pub async fn reload(&self) -> Result<(), String> {
        let mut loaded_forms = HashMap::new();
        let mut flat_items: Vec<(Vec<String>, String, bool)> = Vec::new();

        let catalog = self.catalog_dir.read().await.clone();

        // 1. Scan shipped modules folder
        if catalog.exists() {
            Self::scan_dir_recursive(&catalog, &catalog, false, &mut loaded_forms, &mut flat_items)?;
        }

        // 2. Scan user custom modules folder
        if self.user_forms_dir.exists() {
            Self::scan_dir_recursive(&self.user_forms_dir, &self.user_forms_dir, true, &mut loaded_forms, &mut flat_items)?;
        }

        // Build hierarchical menu tree
        let built_tree = build_menu_hierarchy(flat_items);

        // Update stores
        *self.forms.write().await = loaded_forms;
        *self.menu_tree.write().await = built_tree;

        Ok(())
    }

    fn scan_dir_recursive(
        base_dir: &Path,
        current_dir: &Path,
        is_custom: bool,
        forms_map: &mut HashMap<String, Value>,
        flat_items: &mut Vec<(Vec<String>, String, bool)>,
    ) -> Result<(), String> {
        let entries = std::fs::read_dir(current_dir)
            .map_err(|e| format!("Failed to read directory {}: {e}", current_dir.display()))?;

        // Sort paths alphabetically so menu items and categories are consistently ordered
        let mut paths: Vec<_> = entries.flatten().map(|e| e.path()).collect();
        paths.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

        for path in paths {
            if path.is_dir() {
                Self::scan_dir_recursive(base_dir, &path, is_custom, forms_map, flat_items)?;
            } else if path.file_name().and_then(|n| n.to_str()).map_or(false, |n| n.ends_with(".rform.json")) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(json_val) = serde_json::from_str::<Value>(&content) {
                        if let Some(id) = json_val["id"].as_str() {
                            let form_id = id.to_string();

                            // The Analyze menu follows the folder hierarchy inside the modules directory.
                            // The name of each folder becomes the menu category/submenu label.
                            let mut menu_path = vec!["Analyze".to_string()];

                            if let Ok(rel_path) = path.parent().unwrap_or(base_dir).strip_prefix(base_dir) {
                                for comp in rel_path.components() {
                                    let comp_str = comp.as_os_str().to_string_lossy();
                                    if !comp_str.is_empty() {
                                        menu_path.push(format_folder_name(&comp_str));
                                    }
                                }
                            }

                            // Determine leaf menu item label
                            let leaf_label = get_leaf_label(&json_val, &path);
                            menu_path.push(leaf_label);

                            flat_items.push((menu_path, form_id.clone(), is_custom));
                            forms_map.insert(form_id, json_val);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn get_menu(&self) -> Vec<MenuNode> {
        self.menu_tree.read().await.clone()
    }

    pub async fn get_form(&self, id: &str) -> Option<Value> {
        self.forms.read().await.get(id).cloned()
    }

    pub async fn save_custom_form(&self, form_val: Value) -> Result<String, String> {
        let id = form_val["id"]
            .as_str()
            .ok_or_else(|| "Form schema must have an 'id' field".to_string())?;

        // Determine destination folder based on category (default: "Custom")
        let category = form_val["menu_path"]
            .as_array()
            .and_then(|arr| {
                if arr.len() >= 2 {
                    arr.get(1).and_then(|v| v.as_str())
                } else {
                    None
                }
            })
            .unwrap_or("Custom");

        let target_dir = self.user_forms_dir.join(category);
        std::fs::create_dir_all(&target_dir)
            .map_err(|e| format!("Failed to create user modules directory: {e}"))?;

        let safe_filename = format!("{}.rform.json", id.replace(['/', '\\', ' '], "_"));
        let target_path = target_dir.join(safe_filename);

        let pretty_json = serde_json::to_string_pretty(&form_val)
            .map_err(|e| format!("Failed to serialize form JSON: {e}"))?;

        std::fs::write(&target_path, pretty_json)
            .map_err(|e| format!("Failed to save form file {}: {e}", target_path.display()))?;

        // Hot-reload menu tree
        self.reload().await?;

        Ok(id.to_string())
    }
}

/// Converts a directory name into a human-readable menu label.
/// If the directory name already contains spaces or uppercase letters (e.g. "Compare Means" or "T-Tests"),
/// it is preserved as the user named it.
/// If it is kebab-case or snake_case (e.g. "compare-means" or "binary_logistic"),
/// it is converted into Title Case.
pub fn format_folder_name(name: &str) -> String {
    if name.contains(' ') || name.chars().any(|c| c.is_uppercase()) {
        return name.to_string();
    }
    name.split(|c| c == '-' || c == '_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Determines the leaf menu item label from form schema or file name.
fn get_leaf_label(json_val: &Value, path: &Path) -> String {
    if let Some(lbl) = json_val["menu_label"].as_str() {
        return lbl.to_string();
    }
    if let Some(last_in_path) = json_val["menu_path"].as_array().and_then(|arr| arr.last()).and_then(|v| v.as_str()) {
        return last_in_path.to_string();
    }
    if let Some(title) = json_val["window"]["title"].as_str() {
        if title.ends_with("...") {
            return title.to_string();
        } else {
            return format!("{title}...");
        }
    }
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("Module");
    let clean = stem.strip_suffix(".rform").unwrap_or(stem);
    let title_cased = format_folder_name(clean);
    format!("{title_cased}...")
}

fn find_catalog_dir() -> PathBuf {
    let candidates = [
        PathBuf::from("modules"),
        PathBuf::from("../modules"),
        PathBuf::from("/home/dpng/Projects/r_rad_studio/modules"),
        PathBuf::from("forms-catalog"),
        PathBuf::from("../forms-catalog"),
        PathBuf::from("/home/dpng/Projects/r_rad_studio/forms-catalog"),
    ];

    for c in &candidates {
        if c.exists() {
            return c.clone();
        }
    }
    PathBuf::from("modules")
}

fn get_user_forms_dir() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".config/r-rad-studio/modules")
    } else {
        PathBuf::from("user-modules")
    }
}

fn build_menu_hierarchy(items: Vec<(Vec<String>, String, bool)>) -> Vec<MenuNode> {
    let mut root_nodes: Vec<MenuNode> = Vec::new();

    for (path, form_id, is_custom) in items {
        if path.is_empty() {
            continue;
        }

        let mut current_children = &mut root_nodes;

        for (idx, segment) in path.iter().enumerate() {
            let is_leaf = idx == path.len() - 1;

            let existing_idx = current_children.iter().position(|node| &node.label == segment);

            let node_idx = match existing_idx {
                Some(i) => i,
                None => {
                    let new_node = MenuNode {
                        label: segment.clone(),
                        children: Vec::new(),
                        form_id: if is_leaf { Some(form_id.clone()) } else { None },
                        is_custom: if is_leaf { is_custom } else { false },
                    };
                    current_children.push(new_node);
                    current_children.len() - 1
                }
            };

            if is_leaf {
                current_children[node_idx].form_id = Some(form_id.clone());
                current_children[node_idx].is_custom = is_custom;
                break;
            } else {
                current_children = &mut current_children[node_idx].children;
            }
        }
    }

    root_nodes
}
