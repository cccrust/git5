use crate::object::{git4_dir, hash_object, read_object, write_object};
use anyhow::Result;
use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::io::Write;

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
    fn test_write_tree_single_file() {
        let _temp = setup_test_repo();
        std::fs::write("test.txt", "hello").unwrap();
        let hash = write_tree(Path::new(".")).unwrap();
        assert_eq!(hash.len(), 40);
    }

    #[test]
    fn test_write_tree_empty() {
        let _temp = setup_test_repo();
        let hash = write_tree(Path::new(".")).unwrap();
        assert_eq!(hash.len(), 40);
    }

    #[test]
    fn test_write_tree_nested_dirs() {
        let _temp = setup_test_repo();
        fs::create_dir_all("dir1/dir2").unwrap();
        std::fs::write("dir1/dir2/file.txt", "content").unwrap();
        let hash = write_tree(Path::new(".")).unwrap();
        assert_eq!(hash.len(), 40);
    }

    #[test]
    fn test_restore_tree() {
        let _temp = setup_test_repo();
        std::fs::write("test.txt", "test content").unwrap();
        let tree_hash = write_tree(Path::new(".")).unwrap();
        std::fs::remove_file("test.txt").unwrap();
        restore_tree(&tree_hash, Path::new(".")).unwrap();
        let content = std::fs::read_to_string("test.txt").unwrap();
        assert_eq!(content, "test content");
    }

    #[test]
    fn test_restore_tree_nested() {
        let _temp = setup_test_repo();
        fs::create_dir_all("a/b").unwrap();
        std::fs::write("a/b/file.txt", "nested").unwrap();
        let tree_hash = write_tree(Path::new(".")).unwrap();
        std::fs::remove_dir_all("a").unwrap();
        restore_tree(&tree_hash, Path::new(".")).unwrap();
        assert!(Path::new("a/b/file.txt").exists());
    }

    #[test]
    fn test_read_tree_recursive() {
        let _temp = setup_test_repo();
        std::fs::write("file1.txt", "content1").unwrap();
        std::fs::write("file2.txt", "content2").unwrap();
        let tree_hash = write_tree(Path::new(".")).unwrap();
        let mut files = BTreeMap::new();
        read_tree_recursive(&tree_hash, Path::new(""), &mut files).unwrap();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_tree_ignores_git4() {
        let _temp = setup_test_repo();
        std::fs::write("normal.txt", "normal").unwrap();
        std::fs::write(".git4/somefile", "git4").unwrap();
        let hash = write_tree(Path::new(".")).unwrap();
        let mut files = BTreeMap::new();
        read_tree_recursive(&hash, Path::new(""), &mut files).unwrap();
        assert!(files.contains_key("normal.txt"));
    }

    #[test]
    fn test_get_head_tree_no_commit() {
        let _temp = setup_test_repo();
        let result = get_head_tree().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_tree_sorting() {
        let _temp = setup_test_repo();
        std::fs::write("z_file.txt", "z").unwrap();
        std::fs::write("a_file.txt", "a").unwrap();
        let hash = write_tree(Path::new(".")).unwrap();
        let mut files = BTreeMap::new();
        read_tree_recursive(&hash, Path::new(""), &mut files).unwrap();
        let keys: Vec<&String> = files.keys().collect();
        assert_eq!(keys[0], "a_file.txt");
        assert_eq!(keys[1], "z_file.txt");
    }
}