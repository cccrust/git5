use crate::object::{git4_dir, hash_object};
use anyhow::Result;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub fn read_index() -> Result<BTreeMap<String, String>> {
    let dir = git4_dir()?;
    let index_path = dir.join("index");
    let mut index = BTreeMap::new();
    if index_path.exists() {
        let content = fs::read_to_string(&index_path)?;
        for line in content.lines() {
            if let Some((hash, path)) = line.split_once(' ') {
                index.insert(path.to_string(), hash.to_string());
            }
        }
    }
    Ok(index)
}

pub fn add_files(files: Vec<String>) -> Result<()> {
    let dir = git4_dir()?;
    let index_path = dir.join("index");
    
    let mut index: BTreeMap<String, String> = BTreeMap::new();
    
    if index_path.exists() {
        let content = fs::read_to_string(&index_path)?;
        for line in content.lines() {
            if let Some((hash, path)) = line.split_once(' ') {
                index.insert(path.to_string(), hash.to_string());
            }
        }
    }
    
    for file in files {
        let path = Path::new(&file);
        if path.exists() && path.is_file() {
            let hash = hash_object(&file, true)?;
            println!("Added {} ({})", file, hash);
            index.insert(file.clone(), hash);
        } else {
            println!("Skipping {} (not found or not a regular file)", file);
        }
    }
    
    let mut new_index = String::new();
    for (path, hash) in index {
        new_index.push_str(&format!("{} {}\n", hash, path));
    }
    fs::write(index_path, new_index)?;
    
    Ok(())
}

pub fn get_workspace_files(path: &Path, base: &Path, map: &mut BTreeMap<String, String>) -> Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let name = entry.file_name().into_string().unwrap_or_default();
        if name == ".git4" || name == "target" || name.starts_with('.') || name == "git.sh" {
            continue;
        }
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            get_workspace_files(&entry.path(), base, map)?;
        } else if file_type.is_file() {
            let relative = entry.path().strip_prefix(base).unwrap().to_path_buf();
            let relative_str = relative.to_str().unwrap().to_string();
            let hash = hash_object(entry.path().to_str().unwrap(), false)?;
            map.insert(relative_str, hash);
        }
    }
    Ok(())
}