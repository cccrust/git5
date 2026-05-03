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

#[test]
fn test_branch() {
    let _temp = setup_repo();
    git5().arg("init").output().unwrap();

    std::fs::write("test.txt", "content").unwrap();
    git5().arg("add").arg("test.txt").output().unwrap();
    git5().arg("commit").arg("-m").arg("Initial").output().unwrap();

    let output = git5().arg("branch").arg("test-branch").output().unwrap();
    assert!(output.status.success());

    let output = git5().arg("branch").output().unwrap();
    assert!(String::from_utf8_lossy(&output.stdout).contains("test-branch"));
}

#[test]
fn test_config_set_get() {
    let _temp = setup_repo();
    git5().arg("init").output().unwrap();

    let output = git5().arg("config").arg("user.name").arg("Test User").output().unwrap();
    assert!(output.status.success());

    let output = git5().arg("config").arg("user.name").output().unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Test User"));
}

#[test]
fn test_config_list() {
    let _temp = setup_repo();
    git5().arg("init").output().unwrap();

    git5().arg("config").arg("user.email").arg("test@example.com").output().unwrap();

    let output = git5().arg("config").arg("--list").output().unwrap();
    eprintln!("stdout: {:?}", String::from_utf8_lossy(&output.stdout));
    eprintln!("stderr: {:?}", String::from_utf8_lossy(&output.stderr));
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("user.email"));
}

#[test]
fn test_log() {
    let _temp = setup_repo();
    git5().arg("init").output().unwrap();

    std::fs::write("test.txt", "content").unwrap();
    git5().arg("add").arg("test.txt").output().unwrap();
    git5().arg("commit").arg("-m").arg("First commit").output().unwrap();

    let output = git5().arg("log").output().unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("commit"));
}

#[test]
fn test_checkout_new_branch() {
    let _temp = setup_repo();
    git5().arg("init").output().unwrap();

    std::fs::write("test.txt", "content").unwrap();
    git5().arg("add").arg("test.txt").output().unwrap();
    git5().arg("commit").arg("-m").arg("Initial").output().unwrap();

    let output = git5().arg("checkout").arg("-b").arg("new-branch").output().unwrap();
    eprintln!("stdout: {:?}", String::from_utf8_lossy(&output.stdout));
    eprintln!("stderr: {:?}", String::from_utf8_lossy(&output.stderr));
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("new-branch"));
}

#[test]
fn test_checkout_existing_branch() {
    let _temp = setup_repo();
    git5().arg("init").output().unwrap();

    std::fs::write("test.txt", "content").unwrap();
    git5().arg("add").arg("test.txt").output().unwrap();
    git5().arg("commit").arg("-m").arg("Initial").output().unwrap();
    git5().arg("branch").arg("feature").output().unwrap();

    std::fs::write("test.txt", "v2").unwrap();
    git5().arg("add").arg("test.txt").output().unwrap();
    git5().arg("commit").arg("-m").arg("Update").output().unwrap();

    let output = git5().arg("checkout").arg("feature").output().unwrap();
    assert!(output.status.success());
}

#[test]
fn test_clone_local() {
    let source = TempDir::new().unwrap();
    let dest = TempDir::new().unwrap();
    let _dest_path = dest.as_ref().join("cloned");

    std::env::set_current_dir(source.as_ref()).unwrap();
    git5().arg("init").output().unwrap();
    std::fs::write("test.txt", "content").unwrap();
    git5().arg("add").arg("test.txt").output().unwrap();
    git5().arg("commit").arg("-m").arg("initial").output().unwrap();

    std::env::set_current_dir(dest.as_ref()).unwrap();
    let output = git5()
        .arg("clone")
        .arg(source.as_ref().to_str().unwrap())
        .arg("cloned")
        .output()
        .unwrap();
    assert!(output.status.success(), "clone failed: {:?}", output.stderr);
    assert!(std::path::Path::new("cloned/test.txt").exists());
}

#[test]
fn test_push_fetch() {
    let local = TempDir::new().unwrap();
    let remote = TempDir::new().unwrap();

    std::env::set_current_dir(remote.as_ref()).unwrap();
    git5().arg("init").output().unwrap();

    std::env::set_current_dir(local.as_ref()).unwrap();
    git5().arg("init").output().unwrap();
    std::fs::write("test.txt", "content").unwrap();
    git5().arg("add").arg("test.txt").output().unwrap();
    git5().arg("commit").arg("-m").arg("initial").output().unwrap();

    let output = git5()
        .arg("push")
        .arg(remote.as_ref().to_str().unwrap())
        .arg("main")
        .output()
        .unwrap();
    assert!(output.status.success(), "push failed: {:?}", output.stderr);

    let output = git5()
        .arg("fetch")
        .arg(remote.as_ref().to_str().unwrap())
        .output()
        .unwrap();
    assert!(output.status.success(), "fetch failed: {:?}", output.stderr);
}

#[test]
fn test_tag() {
    let _temp = setup_repo();
    git5().arg("init").output().unwrap();

    std::fs::write("test.txt", "content").unwrap();
    git5().arg("add").arg("test.txt").output().unwrap();
    git5().arg("commit").arg("-m").arg("Initial").output().unwrap();

    let output = git5().arg("tag").arg("v1.0").output().unwrap();
    assert!(output.status.success());

    let output = git5().arg("tag").output().unwrap();
    assert!(String::from_utf8_lossy(&output.stdout).contains("v1.0"));
}

#[test]
fn test_tag_delete() {
    let _temp = setup_repo();
    git5().arg("init").output().unwrap();

    std::fs::write("test.txt", "content").unwrap();
    git5().arg("add").arg("test.txt").output().unwrap();
    git5().arg("commit").arg("-m").arg("Initial").output().unwrap();

    git5().arg("tag").arg("v1.0").output().unwrap();
    let output = git5().arg("tag").arg("-d").arg("v1.0").output().unwrap();
    assert!(output.status.success());

    let output = git5().arg("tag").output().unwrap();
    assert!(!String::from_utf8_lossy(&output.stdout).contains("v1.0"));
}

#[test]
fn test_ls_files() {
    let _temp = setup_repo();
    git5().arg("init").output().unwrap();

    std::fs::write("test.txt", "content").unwrap();
    git5().arg("add").arg("test.txt").output().unwrap();

    let output = git5().arg("ls-files").output().unwrap();
    assert!(String::from_utf8_lossy(&output.stdout).contains("test.txt"));
}

#[test]
fn test_rev_parse() {
    let _temp = setup_repo();
    git5().arg("init").output().unwrap();

    std::fs::write("test.txt", "content").unwrap();
    git5().arg("add").arg("test.txt").output().unwrap();
    git5().arg("commit").arg("-m").arg("Initial").output().unwrap();

    let output = git5().arg("rev-parse").arg("HEAD").output().unwrap();
    eprintln!("stdout: {:?}", String::from_utf8_lossy(&output.stdout));
    eprintln!("stderr: {:?}", String::from_utf8_lossy(&output.stderr));
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim().len(), 40);
}

#[test]
fn test_show_ref() {
    let _temp = setup_repo();
    git5().arg("init").output().unwrap();

    std::fs::write("test.txt", "content").unwrap();
    git5().arg("add").arg("test.txt").output().unwrap();
    git5().arg("commit").arg("-m").arg("Initial").output().unwrap();

    git5().arg("branch").arg("main").output().unwrap();
    git5().arg("tag").arg("v1.0").output().unwrap();

    let output = git5().arg("show-ref").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("refs/heads/main"));
    assert!(stdout.contains("refs/tags/v1.0"));
}

#[test]
fn test_count_objects() {
    let _temp = setup_repo();
    git5().arg("init").output().unwrap();

    std::fs::write("test.txt", "content").unwrap();
    git5().arg("add").arg("test.txt").output().unwrap();
    git5().arg("commit").arg("-m").arg("Initial").output().unwrap();

    let output = git5().arg("count-objects").output().unwrap();
    assert!(output.status.success());
}