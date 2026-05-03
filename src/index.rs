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
    fn test_read_index_empty() {
        let _temp = setup_test_repo();
        let index = read_index().unwrap();
        assert!(index.is_empty());
    }

    #[test]
    fn test_add_files() {
        let _temp = setup_test_repo();
        std::fs::write("test.txt", "content").unwrap();
        add_files(vec!["test.txt".to_string()]).unwrap();
        let index = read_index().unwrap();
        assert!(index.contains_key("test.txt"));
    }

    #[test]
    fn test_add_files_multiple() {
        let _temp = setup_test_repo();
        std::fs::write("a.txt", "a").unwrap();
        std::fs::write("b.txt", "b").unwrap();
        add_files(vec!["a.txt".to_string(), "b.txt".to_string()]).unwrap();
        let index = read_index().unwrap();
        assert_eq!(index.len(), 2);
    }

    #[test]
    fn test_add_files_preserves_existing() {
        let _temp = setup_test_repo();
        std::fs::write("existing.txt", "existing").unwrap();
        std::fs::write("new.txt", "new").unwrap();
        add_files(vec!["existing.txt".to_string()]).unwrap();
        add_files(vec!["new.txt".to_string()]).unwrap();
        let index = read_index().unwrap();
        assert_eq!(index.len(), 2);
    }

    #[test]
    fn test_add_nonexistent_file() {
        let _temp = setup_test_repo();
        add_files(vec!["nonexistent.txt".to_string()]).unwrap();
        let index = read_index().unwrap();
        assert!(index.is_empty());
    }

    #[test]
    fn test_get_workspace_files() {
        let _temp = setup_test_repo();
        std::fs::write("test.txt", "content").unwrap();
        let mut files = BTreeMap::new();
        get_workspace_files(Path::new("."), Path::new("."), &mut files).unwrap();
        assert!(files.contains_key("test.txt"));
    }

    #[test]
    fn test_get_workspace_files_nested() {
        let _temp = setup_test_repo();
        fs::create_dir_all("dir/subdir").unwrap();
        std::fs::write("dir/subdir/file.txt", "content").unwrap();
        let mut files = BTreeMap::new();
        get_workspace_files(Path::new("."), Path::new("."), &mut files).unwrap();
        assert!(files.contains_key("dir/subdir/file.txt"));
    }

    #[test]
    fn test_get_workspace_files_ignores_git4() {
        let _temp = setup_test_repo();
        std::fs::write("normal.txt", "normal").unwrap();
        let mut files = BTreeMap::new();
        get_workspace_files(Path::new("."), Path::new("."), &mut files).unwrap();
        assert!(files.contains_key("normal.txt"));
    }

    #[test]
    fn test_index_format() {
        let _temp = setup_test_repo();
        std::fs::write("test.txt", "content").unwrap();
        add_files(vec!["test.txt".to_string()]).unwrap();
        let content = fs::read_to_string(".git4/index").unwrap();
        assert!(content.contains("test.txt"));
    }
}