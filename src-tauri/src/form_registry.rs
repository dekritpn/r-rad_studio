// src-tauri/src/form_registry.rs
// Discovers, parses, and hot-reloads .rform.json statistical dialog schemas

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
    catalog_dir: PathBuf,
    user_forms_dir: PathBuf,
    forms: Arc<RwLock<HashMap<String, Value>>>,
    menu_tree: Arc<RwLock<Vec<MenuNode>>>,
}

impl FormRegistry {
    pub fn new() -> Self {
        let catalog_dir = find_catalog_dir();
        let user_forms_dir = get_user_forms_dir();
        Self {
            catalog_dir,
            user_forms_dir,
            forms: Arc::new(RwLock::new(HashMap::new())),
            menu_tree: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn reload(&self) -> Result<(), String> {
        let mut loaded_forms = HashMap::new();
        let mut flat_items: Vec<(Vec<String>, String, bool)> = Vec::new();

        // 1. Scan shipped catalog
        if self.catalog_dir.exists() {
            Self::scan_dir_recursive(&self.catalog_dir, false, &mut loaded_forms, &mut flat_items)?;
        }

        // 2. Scan user custom forms
        if self.user_forms_dir.exists() {
            Self::scan_dir_recursive(&self.user_forms_dir, true, &mut loaded_forms, &mut flat_items)?;
        }

        // Build hierarchical menu tree
        let built_tree = build_menu_hierarchy(flat_items);

        // Update stores
        *self.forms.write().await = loaded_forms;
        *self.menu_tree.write().await = built_tree;

        Ok(())
    }

    fn scan_dir_recursive(
        dir: &Path,
        is_custom: bool,
        forms_map: &mut HashMap<String, Value>,
        flat_items: &mut Vec<(Vec<String>, String, bool)>,
    ) -> Result<(), String> {
        let entries = std::fs::read_dir(dir)
            .map_err(|e| format!("Failed to read directory {}: {e}", dir.display()))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                Self::scan_dir_recursive(&path, is_custom, forms_map, flat_items)?;
            } else if path.file_name().and_then(|n| n.to_str()).map_or(false, |n| n.ends_with(".rform.json")) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(json_val) = serde_json::from_str::<Value>(&content) {
                        if let Some(id) = json_val["id"].as_str() {
                            let form_id = id.to_string();
                            let menu_path: Vec<String> = json_val["menu_path"]
                                .as_array()
                                .map(|arr| {
                                    arr.iter()
                                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                        .collect()
                                })
                                .unwrap_or_else(|| vec!["Analyze".to_string(), "Custom".to_string(), form_id.clone()]);

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

        // Ensure user forms dir exists
        std::fs::create_dir_all(&self.user_forms_dir)
            .map_err(|e| format!("Failed to create user forms directory: {e}"))?;

        let safe_filename = format!("{}.rform.json", id.replace(['/', '\\', ' '], "_"));
        let target_path = self.user_forms_dir.join(safe_filename);

        let pretty_json = serde_json::to_string_pretty(&form_val)
            .map_err(|e| format!("Failed to serialize form JSON: {e}"))?;

        std::fs::write(&target_path, pretty_json)
            .map_err(|e| format!("Failed to save form file {}: {e}", target_path.display()))?;

        // Hot-reload menu tree
        self.reload().await?;

        Ok(id.to_string())
    }
}

fn find_catalog_dir() -> PathBuf {
    let candidates = [
        PathBuf::from("forms-catalog"),
        PathBuf::from("../forms-catalog"),
        PathBuf::from("/home/dpng/Projects/r_rad_studio/forms-catalog"),
    ];

    for c in &candidates {
        if c.exists() {
            return c.clone();
        }
    }
    PathBuf::from("forms-catalog")
}

fn get_user_forms_dir() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".config/r-rad-studio/forms")
    } else {
        PathBuf::from("user-forms")
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
