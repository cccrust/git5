use anyhow::{anyhow, Context, Result};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

pub fn git4_dir() -> Result<PathBuf> {
    let mut current = std::env::current_dir()?;
    loop {
        let git4_path = current.join(".git4");
        if git4_path.exists() && git4_path.is_dir() {
            return Ok(git4_path);
        }
        if !current.pop() {
            return Err(anyhow!("Not a git4 repository (or any of the parent directories): .git4"));
        }
    }
}

pub fn read_object(hash: &str) -> Result<(String, Vec<u8>)> {
    let dir = git4_dir()?;
    let obj_path = dir.join("objects").join(&hash[0..2]).join(&hash[2..]);
    
    let compressed = fs::read(&obj_path).with_context(|| format!("Object {} not found", hash))?;
    let mut decoder = ZlibDecoder::new(compressed.as_slice());
    let mut raw = Vec::new();
    decoder.read_to_end(&mut raw)?;

    let nul_pos = raw.iter().position(|&b| b == 0).context("Invalid object format")?;
    let header = String::from_utf8(raw[0..nul_pos].to_vec())?;
    
    let parts: Vec<&str> = header.split(' ').collect();
    let obj_type = parts[0].to_string();
    
    let content = raw[nul_pos + 1..].to_vec();
    Ok((obj_type, content))
}

pub fn write_object(obj_type: &str, content: &[u8]) -> Result<String> {
    let header = format!("{} {}\0", obj_type, content.len());
    let mut store = Vec::new();
    store.extend_from_slice(header.as_bytes());
    store.extend_from_slice(content);

    let mut hasher = Sha1::new();
    hasher.update(&store);
    let hash = hex::encode(hasher.finalize());

    let dir = git4_dir()?;
    let obj_dir = dir.join("objects").join(&hash[0..2]);
    if !obj_dir.exists() {
        fs::create_dir_all(&obj_dir)?;
    }
    let obj_path = obj_dir.join(&hash[2..]);
    
    if !obj_path.exists() {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&store)?;
        let compressed = encoder.finish()?;
        fs::write(obj_path, compressed)?;
    }

    Ok(hash)
}

pub fn hash_object(file: &str, write: bool) -> Result<String> {
    let content = fs::read(file)?;
    if write {
        write_object("blob", &content)
    } else {
        let header = format!("blob {}\0", content.len());
        let mut store = Vec::new();
        store.extend_from_slice(header.as_bytes());
        store.extend_from_slice(&content);
        
        let mut hasher = Sha1::new();
        hasher.update(&store);
        Ok(hex::encode(hasher.finalize()))
    }
}

pub fn cat_file(hash: &str) -> Result<String> {
    let (obj_type, content) = read_object(hash)?;
    if obj_type == "blob" || obj_type == "commit" {
        Ok(String::from_utf8_lossy(&content).to_string())
    } else if obj_type == "tree" {
        let mut out = String::new();
        let mut i = 0;
        while i < content.len() {
            let space_pos = i + content[i..].iter().position(|&b| b == b' ').unwrap_or(0);
            let mode_str = String::from_utf8_lossy(&content[i..space_pos]);

            let nul_pos = space_pos + content[space_pos..].iter().position(|&b| b == 0).unwrap_or(0);
            let name_str = String::from_utf8_lossy(&content[space_pos+1..nul_pos]);

            let sha = hex::encode(&content[nul_pos+1..nul_pos+21]);

            out.push_str(&format!("{} {} {}\n", mode_str, sha, name_str));
            i = nul_pos + 21;
        }
        Ok(out)
    } else {
        Ok(format!("<{} object>", obj_type))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
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
    fn test_write_and_read_blob() {
        let _temp = setup_test_repo();
        let content = b"Hello, World!";
        let hash = write_object("blob", content).unwrap();
        assert_eq!(hash.len(), 40);
        let (obj_type, read_content) = read_object(&hash).unwrap();
        assert_eq!(obj_type, "blob");
        assert_eq!(read_content, content);
    }

    #[test]
    fn test_write_object_twice_same_hash() {
        let _temp = setup_test_repo();
        let content = b"test content";
        let hash1 = write_object("blob", content).unwrap();
        let hash2 = write_object("blob", content).unwrap();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_object_no_write() {
        let _temp = setup_test_repo();
        let temp_dir = std::env::current_dir().unwrap();
        let test_file = temp_dir.join("test.txt");
        std::fs::write(&test_file, "test").unwrap();

        let hash = hash_object("test.txt", false).unwrap();
        assert_eq!(hash.len(), 40);
    }

    #[test]
    fn test_hash_object_with_write() {
        let _temp = setup_test_repo();
        let temp_dir = std::env::current_dir().unwrap();
        let test_file = temp_dir.join("test.txt");
        std::fs::write(&test_file, "test").unwrap();

        let hash = hash_object("test.txt", true).unwrap();
        assert_eq!(hash.len(), 40);
    }

    #[test]
    fn test_cat_file_blob() {
        let _temp = setup_test_repo();
        let content = b"Test content for cat_file";
        let hash = write_object("blob", content).unwrap();
        let output = cat_file(&hash).unwrap();
        assert_eq!(output, "Test content for cat_file");
    }

    #[test]
    fn test_write_tree_object() {
        let _temp = setup_test_repo();
        let content = b"tree content";
        write_object("tree", content).unwrap();
    }

    #[test]
    fn test_write_commit_object() {
        let _temp = setup_test_repo();
        let tree_hash = write_object("tree", b"").unwrap();
        let commit_content = format!("tree {}\n\nInitial commit\n", tree_hash);
        let commit_hash = write_object("commit", commit_content.as_bytes()).unwrap();
        assert_eq!(commit_hash.len(), 40);
    }

    #[test]
    fn test_object_not_found() {
        let _temp = setup_test_repo();
        let result = read_object("0000000000000000000000000000000000000000");
        assert!(result.is_err());
    }
}