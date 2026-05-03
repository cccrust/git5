use crate::object::{git4_dir, read_object, write_object};
use anyhow::{Context, Result};
use chrono::Utc;
use std::fs;

pub fn get_head() -> Result<Option<String>> {
    let dir = git4_dir()?;
    let head_path = dir.join("HEAD");
    if !head_path.exists() {
        return Ok(None);
    }
    
    let head_content = fs::read_to_string(head_path)?;
    let head_content = head_content.trim();
    
    if head_content.starts_with("ref: ") {
        let ref_path = head_content.strip_prefix("ref: ").unwrap().trim();
        let full_ref_path = dir.join(ref_path);
        if full_ref_path.exists() {
            let hash = fs::read_to_string(full_ref_path)?;
            Ok(Some(hash.trim().to_string()))
        } else {
            Ok(None)
        }
    } else {
        Ok(Some(head_content.to_string()))
    }
}

pub fn update_head(hash: &str) -> Result<()> {
    let dir = git4_dir()?;
    let head_path = dir.join("HEAD");
    let head_content = fs::read_to_string(&head_path)?;
    let head_content = head_content.trim();
    
    if head_content.starts_with("ref: ") {
        let ref_path = head_content.strip_prefix("ref: ").unwrap().trim();
        let full_ref_path = dir.join(ref_path);
        fs::write(full_ref_path, format!("{}\n", hash))?;
    } else {
        fs::write(head_path, format!("{}\n", hash))?;
    }
    Ok(())
}

pub fn commit_tree(tree_hash: &str, parent_hash: Option<&str>, message: &str) -> Result<String> {
    let mut content = format!("tree {}\n", tree_hash);
    if let Some(parent) = parent_hash {
        content.push_str(&format!("parent {}\n", parent));
    }
    
    let author = "git4 User <git4@example.com>";
    let timestamp = Utc::now().timestamp();
    let tz = "+0000";
    
    content.push_str(&format!("author {} {} {}\n", author, timestamp, tz));
    content.push_str(&format!("committer {} {} {}\n", author, timestamp, tz));
    content.push_str("\n");
    content.push_str(message);
    content.push_str("\n");

    write_object("commit", content.as_bytes())
}

pub fn resolve_revision(name: &str) -> Result<String> {
    let dir = git4_dir()?;
    let branch_path = dir.join("refs/heads").join(name);
    if branch_path.exists() {
        let hash = fs::read_to_string(branch_path)?;
        return Ok(hash.trim().to_string());
    }
    let obj_path = dir.join("objects").join(&name[0..2]).join(&name[2..]);
    if obj_path.exists() && name.len() >= 40 {
        return Ok(name.to_string());
    }
    Err(anyhow::anyhow!("Cannot resolve revision: {}", name))
}

pub fn get_commit_parent(hash: &str) -> Result<Option<String>> {
    let (obj_type, content) = read_object(hash)?;
    if obj_type == "commit" {
        let content_str = String::from_utf8_lossy(&content);
        for line in content_str.lines() {
            if let Some(p) = line.strip_prefix("parent ") {
                return Ok(Some(p.to_string()));
            }
        }
    }
    Ok(None)
}

pub fn is_ancestor(ancestor: &str, target: &str) -> Result<bool> {
    let mut current = Some(target.to_string());
    while let Some(hash) = current {
        if hash == ancestor {
            return Ok(true);
        }
        current = get_commit_parent(&hash)?;
    }
    Ok(false)
}