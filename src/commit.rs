use crate::error::{Git5Error, Result};
use crate::object::{git4_dir, read_object, write_object};
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

    if name == "HEAD" {
        return get_head()?.ok_or_else(|| Git5Error::InvalidRef("HEAD has no commit".to_string()));
    }

    let branch_path = dir.join("refs/heads").join(name);
    if branch_path.exists() {
        let hash = fs::read_to_string(branch_path)?;
        return Ok(hash.trim().to_string());
    }
    let obj_path = dir.join("objects").join(&name[0..2]).join(&name[2..]);
    if obj_path.exists() && name.len() >= 40 {
        return Ok(name.to_string());
    }
    Err(Git5Error::InvalidRef(format!("Cannot resolve revision: {}", name)))
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn init() -> Result<()> {
        fs::create_dir_all(".git4")?;
        fs::create_dir_all(".git4/objects")?;
        fs::create_dir_all(".git4/refs")?;
        fs::create_dir_all(".git4/refs/heads")?;
        fs::write(".git4/HEAD", "ref: refs/heads/main\n")?;
        Ok(())
    }

    fn setup_test_repo() -> TempDir {
        let temp = TempDir::new().unwrap();
        std::env::set_current_dir(&temp).unwrap();
        init().unwrap();
        temp
    }

    #[test]
    fn test_get_head_no_commits() {
        let _temp = setup_test_repo();
        let head = get_head().unwrap();
        assert!(head.is_none());
    }

    #[test]
    fn test_update_head_ref() {
        let _temp = setup_test_repo();
        let tree_hash = write_object("tree", b"").unwrap();
        let commit_hash = commit_tree(&tree_hash, None, "Initial").unwrap();
        update_head(&commit_hash).unwrap();
        let head = get_head().unwrap();
        assert_eq!(head, Some(commit_hash));
    }

    #[test]
    fn test_commit_tree_basic() {
        let _temp = setup_test_repo();
        let tree_hash = write_object("tree", b"").unwrap();
        let hash = commit_tree(&tree_hash, None, "Test commit").unwrap();
        assert_eq!(hash.len(), 40);
    }

    #[test]
    fn test_commit_tree_with_parent() {
        let _temp = setup_test_repo();
        let tree_hash = write_object("tree", b"").unwrap();
        let parent_hash = commit_tree(&tree_hash, None, "Parent").unwrap();
        let child_hash = commit_tree(&tree_hash, Some(&parent_hash), "Child").unwrap();
        let parent = get_commit_parent(&child_hash).unwrap();
        assert_eq!(parent, Some(parent_hash));
    }

    #[test]
    fn test_resolve_revision_branch() {
        let _temp = setup_test_repo();
        let tree_hash = write_object("tree", b"").unwrap();
        let commit_hash = commit_tree(&tree_hash, None, "Test").unwrap();
        fs::create_dir_all(".git4/refs/heads").unwrap();
        fs::write(".git4/refs/heads/test-branch", format!("{}\n", commit_hash)).unwrap();
        let resolved = resolve_revision("test-branch").unwrap();
        assert_eq!(resolved, commit_hash);
    }

    #[test]
    fn test_resolve_revision_commit() {
        let _temp = setup_test_repo();
        let tree_hash = write_object("tree", b"").unwrap();
        let commit_hash = commit_tree(&tree_hash, None, "Test").unwrap();
        let resolved = resolve_revision(&commit_hash).unwrap();
        assert_eq!(resolved, commit_hash);
    }

    #[test]
    fn test_resolve_revision_not_found() {
        let _temp = setup_test_repo();
        let result = resolve_revision("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_commit_parent_none() {
        let _temp = setup_test_repo();
        let tree_hash = write_object("tree", b"").unwrap();
        let commit_hash = commit_tree(&tree_hash, None, "First").unwrap();
        let parent = get_commit_parent(&commit_hash).unwrap();
        assert!(parent.is_none());
    }

    #[test]
    fn test_is_ancestor_true() {
        let _temp = setup_test_repo();
        let tree_hash = write_object("tree", b"").unwrap();
        let commit1 = commit_tree(&tree_hash, None, "First").unwrap();
        let commit2 = commit_tree(&tree_hash, Some(&commit1), "Second").unwrap();
        let result = is_ancestor(&commit1, &commit2).unwrap();
        assert!(result);
    }

    #[test]
    fn test_is_ancestor_false() {
        let _temp = setup_test_repo();
        let tree_hash = write_object("tree", b"").unwrap();
        let commit1 = commit_tree(&tree_hash, None, "First").unwrap();
        let commit2 = commit_tree(&tree_hash, None, "Second").unwrap();
        let result = is_ancestor(&commit1, &commit2).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_commit_format() {
        let _temp = setup_test_repo();
        let tree_hash = write_object("tree", b"").unwrap();
        let hash = commit_tree(&tree_hash, None, "Test message").unwrap();
        let (_obj_type, content) = read_object(&hash).unwrap();
        let content_str = String::from_utf8_lossy(&content);
        assert!(content_str.contains("tree "));
        assert!(content_str.contains("author "));
        assert!(content_str.contains("committer "));
        assert!(content_str.contains("Test message"));
    }
}