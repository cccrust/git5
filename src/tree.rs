use crate::object::{git4_dir, hash_object, read_object, write_object};
use anyhow::Result;
use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub fn write_tree(path: &Path) -> Result<String> {
    let mut entries = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let file_name = entry.file_name().into_string().unwrap_or_default();
        
        if file_name == ".git4" || file_name == "target" || file_name.starts_with('.') {
            continue;
        }

        if file_type.is_dir() {
            let tree_hash = write_tree(&entry.path())?;
            entries.push((
                "40000".to_string(),
                file_name,
                tree_hash,
            ));
        } else if file_type.is_file() {
            let meta = entry.metadata()?;
            let mode = if meta.permissions().mode() & 0o111 != 0 {
                "100755"
            } else {
                "100644"
            };
            let blob_hash = hash_object(entry.path().to_str().unwrap(), true)?;
            entries.push((mode.to_string(), file_name, blob_hash));
        }
    }

    entries.sort_by(|a, b| a.1.cmp(&b.1));

    let mut tree_content = Vec::new();
    for (mode, name, hash) in entries {
        tree_content.extend_from_slice(format!("{} {}\0", mode, name).as_bytes());
        tree_content.extend_from_slice(&hex::decode(hash)?);
    }

    write_object("tree", &tree_content)
}

pub fn restore_tree(hash: &str, target_path: &Path) -> Result<()> {
    let (obj_type, content) = read_object(hash)?;
    if obj_type != "tree" {
        return Err(anyhow::anyhow!("Expected tree object, got {}", obj_type));
    }
    if !target_path.exists() {
        fs::create_dir_all(target_path)?;
    }
    let mut i = 0;
    while i < content.len() {
        let space_pos = i + content[i..].iter().position(|&b| b == b' ').unwrap_or(0);
        let mode_str = String::from_utf8_lossy(&content[i..space_pos]);
        let nul_pos = space_pos + content[space_pos..].iter().position(|&b| b == 0).unwrap_or(0);
        let name_str = String::from_utf8_lossy(&content[space_pos+1..nul_pos]);
        let sha = hex::encode(&content[nul_pos+1..nul_pos+21]);
        
        let path = target_path.join(name_str.as_ref());
        if mode_str == "40000" {
            restore_tree(&sha, &path)?;
        } else {
            let (b_type, b_content) = read_object(&sha)?;
            if b_type == "blob" {
                fs::write(&path, b_content)?;
                let mut perms = fs::metadata(&path)?.permissions();
                if mode_str == "100755" {
                    perms.set_mode(0o755);
                } else {
                    perms.set_mode(0o644);
                }
                fs::set_permissions(&path, perms)?;
            }
        }
        i = nul_pos + 21;
    }
    Ok(())
}

pub fn get_head_tree() -> Result<Option<String>> {
    let dir = git4_dir()?;
    let head_path = dir.join("HEAD");
    if !head_path.exists() {
        return Ok(None);
    }
    
    let head_content = fs::read_to_string(head_path)?;
    let head_content = head_content.trim();
    
    let hash = if head_content.starts_with("ref: ") {
        let ref_path = head_content.strip_prefix("ref: ").unwrap().trim();
        let full_ref_path = dir.join(ref_path);
        if full_ref_path.exists() {
            fs::read_to_string(full_ref_path)?.trim().to_string()
        } else {
            return Ok(None);
        }
    } else {
        head_content.to_string()
    };
    
    let (obj_type, content) = read_object(&hash)?;
    if obj_type == "commit" {
        let content_str = String::from_utf8_lossy(&content);
        for line in content_str.lines() {
            if let Some(t) = line.strip_prefix("tree ") {
                return Ok(Some(t.to_string()));
            }
        }
    }
    Ok(None)
}

pub fn read_tree_recursive(hash: &str, base_path: &Path, files: &mut BTreeMap<String, String>) -> Result<()> {
    let (obj_type, content) = read_object(hash)?;
    if obj_type != "tree" {
        return Ok(());
    }
    let mut i = 0;
    while i < content.len() {
        let space_pos = i + content[i..].iter().position(|&b| b == b' ').unwrap_or(0);
        let mode_str = String::from_utf8_lossy(&content[i..space_pos]);
        let nul_pos = space_pos + content[space_pos..].iter().position(|&b| b == 0).unwrap_or(0);
        let name_str = String::from_utf8_lossy(&content[space_pos+1..nul_pos]);
        let sha = hex::encode(&content[nul_pos+1..nul_pos+21]);
        
        let mut path = base_path.to_path_buf();
        if base_path.as_os_str().is_empty() {
             path = std::path::PathBuf::from(name_str.as_ref());
        } else {
             path.push(name_str.as_ref());
        }
        
        if mode_str == "40000" {
            read_tree_recursive(&sha, &path, files)?;
        } else {
            files.insert(path.to_str().unwrap().to_string(), sha);
        }
        
        i = nul_pos + 21;
    }
    Ok(())
}