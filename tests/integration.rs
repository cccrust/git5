use std::process::Command;
use tempfile::TempDir;

fn git5() -> Command {
    Command::new(env!("CARGO_BIN_EXE_git5"))
}

fn setup_repo() -> TempDir {
    let temp = TempDir::new().unwrap();
    std::env::set_current_dir(&temp).unwrap();
    temp
}

#[test]
fn test_init() {
    let _temp = setup_repo();
    let output = git5().arg("init").output().unwrap();
    assert!(output.status.success());
    assert!(std::path::Path::new(".git4").exists());
}

#[test]
fn test_hash_object() {
    let _temp = setup_repo();
    git5().arg("init").output().unwrap();

    std::fs::write("test.txt", "hello world").unwrap();
    let output = git5().arg("hash-object").arg("-w").arg("test.txt").output().unwrap();
    assert!(output.status.success());
    let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(hash.len(), 40);
}

#[test]
fn test_write_tree() {
    let _temp = setup_repo();
    git5().arg("init").output().unwrap();

    std::fs::write("file.txt", "content").unwrap();
    let output = git5().arg("write-tree").output().unwrap();
    assert!(output.status.success());
    let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(hash.len(), 40);
}

#[test]
fn test_commit() {
    let _temp = setup_repo();
    let init_out = git5().arg("init").output().unwrap();
    assert!(init_out.status.success(), "init failed: {:?}", init_out.stderr);

    std::fs::write("test.txt", "initial").unwrap();
    git5().arg("add").arg("test.txt").output().unwrap();

    let output = git5().arg("commit").arg("-m").arg("Initial commit").output().unwrap();
    assert!(output.status.success(), "commit failed: {:?}", output.stderr);
    assert!(String::from_utf8_lossy(&output.stdout).contains("Committed"));
}

#[test]
fn test_status_empty() {
    let _temp = setup_repo();
    git5().arg("init").output().unwrap();

    let output = git5().arg("status").output().unwrap();
    assert!(output.status.success());
}

#[test]
fn test_status_untracked() {
    let _temp = setup_repo();
    git5().arg("init").output().unwrap();

    std::fs::write("new.txt", "content").unwrap();
    let output = git5().arg("status").output().unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("new.txt"));
}