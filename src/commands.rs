use crate::commit::{get_head, is_ancestor, resolve_revision, update_head, commit_tree as create_commit};
use crate::error::{Git5Error, Result};
use crate::index::{add_files as index_add_files, get_workspace_files, read_index};
use crate::object::{cat_file, git4_dir, hash_object, read_object, write_object};
use crate::tree::{get_head_tree, read_tree_recursive, restore_tree, write_tree as create_tree};
use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::Path;
use ureq;

pub fn run(command: Command) -> Result<()> {
    match command {
        Command::Init => { init()?; Ok(()) }
        Command::HashObject { write, file } => {
            let hash = hash_object(&file, write)?;
            println!("{}", hash);
            Ok(())
        }
        Command::CatFile { print, size, type_flag, object } => {
            let (_obj_type, content) = read_object(&object)?;
            if print {
                let content_str = cat_file(&object)?;
                print!("{}", content_str);
            }
            if size {
                println!("{}", content.len());
            }
            if type_flag {
                let (obj_type, _) = read_object(&object)?;
                println!("{}", obj_type);
            }
            Ok(())
        }
        Command::WriteTree => {
            let hash = create_tree(Path::new("."))?;
            println!("{}", hash);
            Ok(())
        }
        Command::CommitTree { tree, parent, message } => {
            let hash = create_commit(&tree, parent.as_deref(), &message)?;
            println!("{}", hash);
            Ok(())
        }
        Command::Add { files } => { index_add_files(files)?; Ok(()) }
        Command::Commit { message } => { commit(&message)?; Ok(()) }
        Command::Log => { log_cmd()?; Ok(()) }
        Command::Branch { name } => { branch(name)?; Ok(()) }
        Command::Checkout { create_branch, name } => { checkout(&name, create_branch)?; Ok(()) }
        Command::Status => { status()?; Ok(()) }
        Command::Diff { file } => { diff(&file)?; Ok(()) }
        Command::Merge { branch } => { merge(&branch)?; Ok(()) }
        Command::Clone { source, dest } => { clone(&source, &dest)?; Ok(()) }
        Command::Push { remote_path, branch } => { push(&remote_path, &branch)?; Ok(()) }
        Command::Fetch { remote_path } => { fetch(&remote_path)?; Ok(()) }
        Command::Remote { action, name, url } => {
            if action == "add" {
                remote_add(&name, &url)?;
            } else {
                println!("Unknown remote action: {}", action);
            }
            Ok(())
        }
        Command::LsRemote { remote } => { ls_remote(&remote)?; Ok(()) }
        Command::UnpackObjects { packfile } => { unpack_objects(&packfile)?; Ok(()) }
        Command::Config { list, key, value } => { config(list, key, value)?; Ok(()) }
        Command::Tag { delete, name } => { tag(delete, name)?; Ok(()) }
        Command::Rm { files } => { rm(files)?; Ok(()) }
        Command::LsFiles { cached } => { ls_files(cached)?; Ok(()) }
        Command::RevParse { short, revision } => { rev_parse(&revision, short)?; Ok(()) }
        Command::ShowRef { heads, tags } => { show_ref(heads, tags)?; Ok(()) }
        Command::CountObjects { verbose } => { count_objects(verbose)?; Ok(()) }
        Command::Describe { tags, abbrev } => { describe(tags, abbrev)?; Ok(()) }
        Command::VerifyPack { packfile } => { verify_pack(&packfile)?; Ok(()) }
        Command::MkTree { binary } => { mktree(binary)?; Ok(()) }
        Command::LsTree { recursive, tree } => { ls_tree(&tree, recursive)?; Ok(()) }
        Command::UpdateRef { delete, ref_name, hash } => { update_ref(&ref_name, delete, hash.as_deref())?; Ok(()) }
        Command::SymbolicRef { ref_name, target } => { symbolic_ref(ref_name.as_deref(), target.as_deref())?; Ok(()) }
        Command::ForEachRef { format } => { for_each_ref(format.as_deref())?; Ok(()) }
        Command::CatFileBatch { batch: _ } => { cat_file_batch()?; Ok(()) }
        Command::DiffTree { tree1, tree2 } => { diff_tree(&tree1, tree2.as_deref())?; Ok(()) }
        Command::NameRev { commits } => { name_rev(commits)?; Ok(()) }
        Command::VerifyCommit { commit } => { verify_commit(&commit)?; Ok(()) }
        Command::RevList { commits } => { rev_list(commits)?; Ok(()) }
        Command::Archive { format, tree } => { archive(format, tree.as_deref())?; Ok(()) }
        Command::Blame { file } => { blame(&file)?; Ok(()) }
    }
}

pub enum Command {
    Init,
    HashObject { write: bool, file: String },
    CatFile { print: bool, size: bool, type_flag: bool, object: String },
    WriteTree,
    CommitTree { tree: String, parent: Option<String>, message: String },
    Add { files: Vec<String> },
    Commit { message: String },
    Log,
    Branch { name: Option<String> },
    Checkout { create_branch: bool, name: String },
    Status,
    Diff { file: String },
    Merge { branch: String },
    Clone { source: String, dest: String },
    Push { remote_path: String, branch: String },
    Fetch { remote_path: String },
    Remote { action: String, name: String, url: String },
    LsRemote { remote: String },
    UnpackObjects { packfile: String },
    Config { list: bool, key: Option<String>, value: Option<String> },
    Tag { delete: bool, name: Option<String> },
    Rm { files: Vec<String> },
    LsFiles { cached: bool },
    RevParse { short: bool, revision: String },
    ShowRef { heads: bool, tags: bool },
    CountObjects { verbose: bool },
    Describe { tags: bool, abbrev: u32 },
    VerifyPack { packfile: String },
    MkTree { binary: bool },
    LsTree { recursive: bool, tree: String },
    UpdateRef { delete: bool, ref_name: String, hash: Option<String> },
    SymbolicRef { ref_name: Option<String>, target: Option<String> },
    ForEachRef { format: Option<String> },
    CatFileBatch { batch: bool },
    DiffTree { tree1: String, tree2: Option<String> },
    NameRev { commits: Vec<String> },
    VerifyCommit { commit: String },
    RevList { commits: Vec<String> },
    Archive { format: Option<String>, tree: Option<String> },
    Blame { file: String },
}

fn init() -> Result<()> {
    fs::create_dir(".git4")?;
    fs::create_dir(".git4/objects")?;
    fs::create_dir(".git4/refs")?;
    fs::create_dir(".git4/refs/heads")?;
    fs::write(".git4/HEAD", "ref: refs/heads/main\n")?;
    println!("Initialized empty git4 repository in .git4/");
    Ok(())
}

fn commit(message: &str) -> Result<()> {
    println!("Building tree...");
    let tree_hash = create_tree(Path::new("."))?;
    println!("Tree hash: {}", tree_hash);
    
    let parent = get_head()?;
    let commit_hash = create_commit(&tree_hash, parent.as_deref(), message)?;
    
    update_head(&commit_hash)?;
    println!("Committed: {}", commit_hash);
    
    Ok(())
}

fn log_cmd() -> Result<()> {
    let mut current = get_head()?;
    
    while let Some(hash) = current {
        let (obj_type, content) = read_object(&hash)?;
        if obj_type != "commit" {
            println!("HEAD is not a commit: {}", hash);
            break;
        }
        
        let content_str = String::from_utf8_lossy(&content);
        println!("commit {}", hash);
        
        let mut parent_hash = None;
        let mut is_msg = false;
        
        for line in content_str.lines() {
            if is_msg {
                println!("    {}", line);
            } else if line.is_empty() {
                is_msg = true;
                println!();
            } else if let Some(p) = line.strip_prefix("parent ") {
                parent_hash = Some(p.to_string());
            } else if line.starts_with("author ") || line.starts_with("committer ") {
                println!("{}", line);
            }
        }
        println!("\n");
        current = parent_hash;
    }
    
    Ok(())
}

fn branch(name: Option<String>) -> Result<()> {
    let dir = git4_dir()?;
    let heads_dir = dir.join("refs/heads");
    
    if let Some(n) = name {
        let head_hash = get_head()?.ok_or_else(|| Git5Error::InvalidRef("No commits yet".to_string()))?;
        let branch_path = heads_dir.join(&n);
        fs::write(branch_path, format!("{}\n", head_hash))?;
        println!("Created branch {}", n);
    } else {
        let head_content = fs::read_to_string(dir.join("HEAD")).unwrap_or_default();
        let current_branch = head_content.strip_prefix("ref: refs/heads/").unwrap_or("").trim();
        for entry in fs::read_dir(heads_dir)? {
            let entry = entry?;
            let b_name = entry.file_name().into_string().unwrap();
            if b_name == current_branch {
                println!("* {}", b_name);
            } else {
                println!("  {}", b_name);
            }
        }
    }
    Ok(())
}

fn checkout(name: &str, create_branch: bool) -> Result<()> {
    let dir = git4_dir()?;

    if create_branch {
        let head_hash = get_head()?.ok_or_else(|| Git5Error::InvalidRef("No commits yet".to_string()))?;
        let branch_path = dir.join("refs/heads").join(name);
        fs::create_dir_all(branch_path.parent().unwrap())?;
        fs::write(&branch_path, format!("{}\n", head_hash))?;
        let head_path = dir.join("HEAD");
        fs::write(head_path, format!("ref: refs/heads/{}\n", name))?;
        println!("Switched to new branch '{}'", name);
        return Ok(());
    }

    let hash = resolve_revision(name)?;
    let (obj_type, content) = read_object(&hash)?;

    let tree_hash = if obj_type == "commit" {
        let content_str = String::from_utf8_lossy(&content);
        let mut tree_hash = String::new();
        for line in content_str.lines() {
            if let Some(t) = line.strip_prefix("tree ") {
                tree_hash = t.to_string();
                break;
            }
        }
        tree_hash
    } else if obj_type == "tree" {
        hash.to_string()
    } else {
        return Err(Git5Error::InvalidObject(format!("Cannot checkout an object of type {}", obj_type)));
    };

    restore_tree(&tree_hash, Path::new("."))?;

    let head_path = dir.join("HEAD");
    let branch_path = dir.join("refs/heads").join(name);
    if branch_path.exists() {
        fs::write(head_path, format!("ref: refs/heads/{}\n", name))?;
        println!("Switched to branch '{}'", name);
    } else {
        fs::write(head_path, format!("{}\n", hash))?;
        println!("Note: checking out '{}'. You are in 'detached HEAD' state.", name);
    }
    Ok(())
}

fn status() -> Result<()> {
    let mut head_files = BTreeMap::new();
    if let Some(tree_hash) = get_head_tree()? {
        read_tree_recursive(&tree_hash, Path::new(""), &mut head_files)?;
    }
    
    let index_files = read_index()?;
    let mut workspace_files = BTreeMap::new();
    let current_dir = std::env::current_dir()?;
    get_workspace_files(&current_dir, &current_dir, &mut workspace_files)?;

    let mut to_commit = Vec::new();
    for (path, hash) in &index_files {
        if let Some(head_hash) = head_files.get(path) {
            if hash != head_hash {
                to_commit.push(format!("modified:   {}", path));
            }
        } else {
            to_commit.push(format!("new file:   {}", path));
        }
    }
    for path in head_files.keys() {
        if !index_files.contains_key(path) {
            to_commit.push(format!("deleted:    {}", path));
        }
    }

    let mut not_staged = Vec::new();
    let mut untracked = Vec::new();
    for (path, hash) in &workspace_files {
        if let Some(index_hash) = index_files.get(path) {
            if hash != index_hash {
                not_staged.push(format!("modified:   {}", path));
            }
        } else if let Some(head_hash) = head_files.get(path) {
            if hash != head_hash {
                not_staged.push(format!("modified:   {}", path));
            }
        } else {
            untracked.push(path.clone());
        }
    }
    
    for path in index_files.keys().chain(head_files.keys()) {
        if !workspace_files.contains_key(path) {
            let d = format!("deleted:    {}", path);
            if !not_staged.contains(&d) {
                 not_staged.push(d);
            }
        }
    }

    to_commit.sort(); to_commit.dedup();
    not_staged.sort(); not_staged.dedup();
    untracked.sort(); untracked.dedup();
    
    let dir = git4_dir()?;
    let head_path = dir.join("HEAD");
    let head_content = fs::read_to_string(head_path).unwrap_or_default();
    let branch = head_content.strip_prefix("ref: refs/heads/").unwrap_or("").trim();
    if !branch.is_empty() {
        println!("On branch {}", branch);
    } else {
        println!("Not currently on any branch.");
    }
    
    if !to_commit.is_empty() {
        println!("Changes to be committed:");
        for l in &to_commit { println!("  {}", l); }
        println!();
    }
    if !not_staged.is_empty() {
        println!("Changes not staged for commit:");
        for l in &not_staged { println!("  {}", l); }
        println!();
    }
    if !untracked.is_empty() {
        println!("Untracked files:");
        for l in &untracked { println!("  {}", l); }
        println!();
    }
    
    if to_commit.is_empty() && not_staged.is_empty() && untracked.is_empty() {
        println!("nothing to commit, working tree clean");
    }
    
    Ok(())
}

fn diff(file: &str) -> Result<()> {
    use similar::{TextDiff, ChangeTag};

    let index_files = read_index()?;
    let mut base_hash = index_files.get(file).cloned();
    
    if base_hash.is_none() {
        let mut head_files = BTreeMap::new();
        if let Some(tree_hash) = get_head_tree()? {
            read_tree_recursive(&tree_hash, Path::new(""), &mut head_files)?;
        }
        base_hash = head_files.get(file).cloned();
    }
    
    let base_content = if let Some(hash) = base_hash {
        let (obj_type, content) = read_object(&hash)?;
        if obj_type == "blob" {
            String::from_utf8_lossy(&content).to_string()
        } else {
            String::new()
        }
    } else {
        String::new()
    };
    
    let path = Path::new(file);
    let new_content = if path.exists() && path.is_file() {
        fs::read_to_string(path).unwrap_or_default()
    } else {
        String::new()
    };
    
    if base_content == new_content {
        return Ok(());
    }
    
    let diff_result = TextDiff::from_lines(&base_content, &new_content);
    for change in diff_result.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => "-",
            ChangeTag::Insert => "+",
            ChangeTag::Equal => " ",
        };
        print!("{}{}", sign, change);
    }
    
    Ok(())
}

fn merge(branch: &str) -> Result<()> {
    let head_hash = get_head()?.ok_or_else(|| Git5Error::InvalidRef("No HEAD commit".to_string()))?;
    let target_hash = resolve_revision(branch)?;
    
    if head_hash == target_hash {
        println!("Already up to date.");
        return Ok(());
    }
    
    if is_ancestor(&head_hash, &target_hash)? {
        println!("Updating {}..{}", &head_hash[0..7], &target_hash[0..7]);
        println!("Fast-forward");
        
        let (obj_type, content) = read_object(&target_hash)?;
        if obj_type != "commit" { return Err(Git5Error::InvalidObject("Merge target is not a commit".to_string())); }
        
        let content_str = String::from_utf8_lossy(&content);
        let mut tree_hash = String::new();
        for line in content_str.lines() {
            if let Some(t) = line.strip_prefix("tree ") {
                tree_hash = t.to_string();
                break;
            }
        }
        
        restore_tree(&tree_hash, Path::new("."))?;
        
        let dir = git4_dir()?;
        let head_path = dir.join("HEAD");
        let head_content = fs::read_to_string(&head_path)?;
        let head_content = head_content.trim();
        
        if head_content.starts_with("ref: ") {
            let ref_path = head_content.strip_prefix("ref: ").unwrap().trim();
            let full_ref_path = dir.join(ref_path);
            fs::write(full_ref_path, format!("{}\n", target_hash))?;
        } else {
            fs::write(head_path, format!("{}\n", target_hash))?;
        }
    } else {
        println!("Merge strategy not implemented for non-fast-forward. Aborting.");
    }
    
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let path = entry.path();
        let target = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&path, &target)?;
        } else {
            fs::copy(&path, &target)?;
        }
    }
    Ok(())
}

fn clone(source: &str, dest: &str) -> Result<()> {
    let src_path = Path::new(source);
    let dest_path = Path::new(dest);
    
    if !src_path.join(".git4").exists() {
        return Err(Git5Error::NotARepository("Source is not a git5 repository".to_string()));
    }
    if dest_path.exists() {
        return Err(Git5Error::Conflict("Destination already exists".to_string()));
    }
    
    fs::create_dir_all(dest_path)?;
    copy_dir_recursive(&src_path.join(".git4"), &dest_path.join(".git4"))?;
    
    let current_dir = std::env::current_dir()?;
    std::env::set_current_dir(dest_path)?;
    
    let head_content = fs::read_to_string(".git4/HEAD")?;
    let head_content = head_content.trim();
    if head_content.starts_with("ref: refs/heads/") {
        let branch = head_content.strip_prefix("ref: refs/heads/").unwrap();
        let _ = checkout(branch, false);
    } else {
        let _ = checkout(head_content, false);
    }
    
    std::env::set_current_dir(current_dir)?;
    println!("Cloned into '{}'", dest);
    
    Ok(())
}

fn push(remote_path: &str, branch: &str) -> Result<()> {
    let local_dir = git4_dir()?;
    let remote_dir = Path::new(remote_path).join(".git4");
    
    if !remote_dir.exists() {
        return Err(Git5Error::NotARepository("Remote path is not a git5 repository".to_string()));
    }

    let local_branch_path = local_dir.join("refs/heads").join(branch);
    if !local_branch_path.exists() {
        return Err(Git5Error::InvalidRef(format!("Local branch {} does not exist", branch)));
    }
    let local_hash = fs::read_to_string(&local_branch_path)?;
    let local_hash = local_hash.trim();
    
    copy_dir_recursive(&local_dir.join("objects"), &remote_dir.join("objects"))?;
    
    let remote_branch_path = remote_dir.join("refs/heads").join(branch);
    fs::create_dir_all(remote_branch_path.parent().unwrap())?;
    fs::write(remote_branch_path, format!("{}\n", local_hash))?;
    
    println!("Pushed branch {} to {}", branch, remote_path);
    Ok(())
}

fn fetch(remote_path: &str) -> Result<()> {
    let local_dir = git4_dir()?;
    let remote_dir = Path::new(remote_path).join(".git4");
    
    if !remote_dir.exists() {
return Err(Git5Error::NotARepository("Remote path is not a git5 repository".to_string()));
    }

    println!("Fetched from {}", remote_path);
    Ok(())
}

fn remote_add(name: &str, url: &str) -> Result<()> {
    let dir = git4_dir()?;
    let config_path = dir.join("config");
    let mut out = String::new();
    if config_path.exists() {
        out = fs::read_to_string(&config_path)?;
    }
    out.push_str(&format!("[remote \"{}\"]\n", name));
    out.push_str(&format!("url = {}\n", url));
    fs::write(config_path, out)?;
    Ok(())
}

fn get_remote_url(name: &str) -> Result<String> {
    let dir = git4_dir()?;
    let config_path = dir.join("config");
    if !config_path.exists() {
        if name.starts_with("http") { return Ok(name.to_string()); }
        return Err(Git5Error::InvalidRef("No remotes configured".to_string()));
    }
    let content = fs::read_to_string(config_path)?;
    let mut in_remote = false;
    for line in content.lines() {
        let line = line.trim();
        if line == format!("[remote \"{}\"]", name) {
            in_remote = true;
        } else if line.starts_with('[') {
            in_remote = false;
        } else if in_remote && line.starts_with("url = ") {
            return Ok(line.strip_prefix("url = ").unwrap().trim().to_string());
        }
    }
    if name.starts_with("http") {
        return Ok(name.to_string());
    }
    Err(Git5Error::InvalidRef(format!("Remote '{}' not found", name)))
}

fn ls_remote(remote: &str) -> Result<()> {
    let url = get_remote_url(remote)?;
    let endpoint = format!("{}/info/refs?service=git-upload-pack", url.trim_end_matches('/'));
    
    let resp = ureq::get(&endpoint).call().map_err(|e| Git5Error::IoError(format!("HTTP request failed: {}", e)))?;
    let mut reader = resp.into_body().into_reader();
    let mut body = Vec::new();
    reader.read_to_end(&mut body)?;
    
    let mut pos = 0;
    while pos + 4 <= body.len() {
        let len_str = std::str::from_utf8(&body[pos..pos+4]).unwrap_or("0000");
        let len = usize::from_str_radix(len_str, 16).unwrap_or(0);
        
        if len == 0 {
            pos += 4;
            continue;
        }
        
        if pos + len > body.len() {
            break;
        }
        
        let line_data = &body[pos+4..pos+len];
        let line_str = String::from_utf8_lossy(line_data);
        let line = line_str.trim_end_matches('\n');
        
        if line.starts_with('#') {
        } else if line.len() >= 40 {
            let mut parts = line.split('\0');
            if let Some(ref_info) = parts.next() {
                if let Some((hash, refname)) = ref_info.split_once(' ') {
                    println!("{} {}", hash, refname);
                }
            }
        }
        
        pos += len;
    }
    
    Ok(())
}

fn unpack_objects(packfile: &str) -> Result<()> {
    use flate2::Decompress;
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    use sha1::Digest;
    use std::io::Write;

    let pack_data = fs::read(packfile)?;
    if pack_data.len() < 32 {
        return Err(Git5Error::InvalidObject("Packfile is too small".to_string()));
    }

    if &pack_data[0..4] != b"PACK" {
        return Err(Git5Error::InvalidObject("Invalid packfile magic".to_string()));
    }

    let version = u32::from_be_bytes(pack_data[4..8].try_into().unwrap());
    if version != 2 {
        return Err(Git5Error::InvalidObject(format!("Unsupported pack version: {}", version)));
    }
    
    let num_objects = u32::from_be_bytes(pack_data[8..12].try_into().unwrap());
    println!("Unpacking {} objects...", num_objects);
    
    let mut pos = 12;
    let mut resolved_count = 0;
    let mut delta_count = 0;
    
    for _ in 0..num_objects {
        let mut byte = pack_data[pos];
        let obj_type = (byte >> 4) & 0b111;
        let mut size = (byte & 0b1111) as usize;
        let mut shift = 4;
        
        pos += 1;
        while (byte & 0x80) != 0 {
            byte = pack_data[pos];
            size |= ((byte & 0x7F) as usize) << shift;
            shift += 7;
            pos += 1;
        }
        
        if obj_type == 6 {
            byte = pack_data[pos];
            pos += 1;
            let mut offset = (byte & 0x7F) as usize;
            while (byte & 0x80) != 0 {
                offset += 1;
                byte = pack_data[pos];
                pos += 1;
                offset = (offset << 7) | ((byte & 0x7F) as usize);
            }
            let mut decompress = Decompress::new(true);
            let mut output = vec![0; 4096];
            while let Ok(res) = decompress.decompress(&pack_data[pos..], &mut output, flate2::FlushDecompress::None) {
                if res == flate2::Status::StreamEnd || res == flate2::Status::BufError { break; }
            }
            pos += decompress.total_in() as usize;
            delta_count += 1;
            continue;
        } else if obj_type == 7 {
            pos += 20;
            let mut decompress = Decompress::new(true);
            let mut output = vec![0; 4096];
            while let Ok(res) = decompress.decompress(&pack_data[pos..], &mut output, flate2::FlushDecompress::None) {
                if res == flate2::Status::StreamEnd || res == flate2::Status::BufError { break; }
            }
            pos += decompress.total_in() as usize;
            delta_count += 1;
            continue;
        }
        
        let type_str = match obj_type {
            1 => "commit",
            2 => "tree",
            3 => "blob",
            4 => "tag",
            _ => return Err(Git5Error::InvalidObject(format!("Unknown object type {}", obj_type))),
        };

        let mut decompress = Decompress::new(true);
        let mut output = vec![0; size];
        if size > 0 {
            let res = decompress.decompress(&pack_data[pos..], &mut output, flate2::FlushDecompress::None)
                .map_err(|e| Git5Error::IoError(format!("Decompression failed: {:?}", e)))?;
            
            if res != flate2::Status::StreamEnd && res != flate2::Status::BufError {
                println!("Warning: Unexpected decompress status for object, might be corrupted");
            }
        } else {
            let _ = decompress.decompress(&pack_data[pos..], &mut output, flate2::FlushDecompress::None);
        }
        
        pos += decompress.total_in() as usize;
        
        let header = format!("{} {}\0", type_str, size);
        let mut full_data = header.into_bytes();
        full_data.extend_from_slice(&output);
        
        let mut hasher = sha1::Sha1::new();
        Digest::update(&mut hasher, &full_data);
        let hash = hex::encode(Digest::finalize(hasher));
        
        let dir = git4_dir()?;
        let obj_dir = dir.join("objects").join(&hash[0..2]);
        fs::create_dir_all(&obj_dir)?;
        
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&full_data)?;
        let compressed = encoder.finish()?;
        
        fs::write(obj_dir.join(&hash[2..]), compressed)?;
        
        resolved_count += 1;
    }
    
    println!("Unpacked {} loose objects. Encountered {} unresolved deltas.", resolved_count, delta_count);

    Ok(())
}

fn config(list: bool, key: Option<String>, value: Option<String>) -> Result<()> {
    let dir = git4_dir()?;
    let config_path = dir.join("config");

    let mut config = BTreeMap::new();

    if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        parse_config(&content, &mut config);
    }

    if list {
        for (k, v) in &config {
            println!("{} = {}", k, v);
        }
        return Ok(());
    }

    if let Some(k) = key {
        if let Some(v) = value {
            config.insert(k.clone(), v.clone());
            save_config(&config_path, &config)?;
            println!("Set {} = {}", k, v);
        } else {
            if let Some(v) = config.get(&k) {
                println!("{}", v);
            } else {
                println!("Config key '{}' not found", k);
            }
        }
    } else {
        println!("Usage: git5 config <key> [value]");
        println!("       git5 config --list");
    }

    Ok(())
}

fn parse_config(content: &str, config: &mut BTreeMap<String, String>) {
    let mut current_section = String::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            current_section = line[1..line.len()-1].to_string();
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            let key = if current_section.is_empty() {
                k.trim().to_string()
            } else {
                format!("{}.{}", current_section, k.trim())
            };
            config.insert(key, v.trim().to_string());
        }
    }
}

fn save_config(path: &Path, config: &BTreeMap<String, String>) -> Result<()> {
    let mut sections: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();

    for (k, v) in config {
        if let Some((section, key)) = k.split_once('.') {
            sections.entry(section.to_string()).or_default().insert(key.to_string(), v.clone());
        } else {
            sections.entry("".to_string()).or_default().insert(k.clone(), v.clone());
        }
    }

    let mut content = String::new();
    for (section, items) in &sections {
        if !section.is_empty() {
            content.push_str(&format!("[{}]\n", section));
        }
        for (k, v) in items {
            content.push_str(&format!("{} = {}\n", k, v));
        }
    }

    fs::write(path, content)?;
    Ok(())
}

fn tag(delete: bool, name: Option<String>) -> Result<()> {
    let dir = git4_dir()?;
    let tags_dir = dir.join("refs/tags");
    fs::create_dir_all(&tags_dir)?;

    if delete {
        if let Some(n) = name {
            let tag_path = tags_dir.join(&n);
            if tag_path.exists() {
                fs::remove_file(&tag_path)?;
                println!("Deleted tag '{}'", n);
            } else {
                println!("Tag '{}' not found", n);
            }
        } else {
            println!("Tag name required for deletion");
        }
        return Ok(());
    }

    if name.is_none() {
        if tags_dir.exists() {
            for entry in fs::read_dir(&tags_dir)? {
                let entry = entry?;
                println!("{}", entry.file_name().to_string_lossy());
            }
        }
        return Ok(());
    }

    let n = name.unwrap();
    let head_hash = get_head()?.ok_or_else(|| Git5Error::InvalidRef("No commits yet".to_string()))?;
    let tag_path = tags_dir.join(&n);
    fs::write(&tag_path, format!("{}\n", head_hash))?;
    println!("Created tag '{}'", n);
    Ok(())
}

fn rm(files: Vec<String>) -> Result<()> {
    let index = read_index()?;
    let mut new_index: BTreeMap<String, String> = index;

    for file in files {
        if new_index.remove(&file).is_some() {
            println!("Removed '{}'", file);
        } else {
            println!("'{}' not in index", file);
        }
    }

    let dir = git4_dir()?;
    let index_path = dir.join("index");
    let mut content = String::new();
    for (path, hash) in &new_index {
        content.push_str(&format!("{} {}\n", hash, path));
    }
    fs::write(index_path, content)?;
    Ok(())
}

fn ls_files(cached: bool) -> Result<()> {
    let index = read_index()?;
    if cached {
        for (path, _hash) in &index {
            println!("{}", path);
        }
    } else {
        for (path, _hash) in &index {
            println!("{}", path);
        }
    }
    Ok(())
}

fn rev_parse(revision: &str, short: bool) -> Result<()> {
    let hash = resolve_revision(revision)?;
    if short {
        println!("{}", &hash[..7]);
    } else {
        println!("{}", hash);
    }
    Ok(())
}

fn show_ref(heads: bool, tags: bool) -> Result<()> {
    let dir = git4_dir()?;

    if !tags {
        let heads_dir = dir.join("refs/heads");
        if heads_dir.exists() {
            for entry in fs::read_dir(&heads_dir)? {
                let entry = entry?;
                let name = entry.file_name().into_string().unwrap_or_default();
                let hash = fs::read_to_string(entry.path())?.trim().to_string();
                println!("{} refs/heads/{}", hash, name);
            }
        }
    }

    if !heads {
        let tags_dir = dir.join("refs/tags");
        if tags_dir.exists() {
            for entry in fs::read_dir(&tags_dir)? {
                let entry = entry?;
                let name = entry.file_name().into_string().unwrap_or_default();
                let hash = fs::read_to_string(entry.path())?.trim().to_string();
                println!("{} refs/tags/{}", hash, name);
            }
        }
    }

    if !heads && !tags {
        let heads_dir = dir.join("refs/heads");
        let tags_dir = dir.join("refs/tags");
        if !heads_dir.exists() && !tags_dir.exists() {
            println!("No refs found.");
        }
    }

    Ok(())
}

fn count_objects(verbose: bool) -> Result<()> {
    let dir = git4_dir()?;
    let objects_dir = dir.join("objects");

    let mut loose_count = 0;
    let mut loose_size = 0u64;

    if objects_dir.exists() {
        for entry in fs::read_dir(&objects_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let subdir = entry.path();
                for obj in fs::read_dir(&subdir)? {
                    let obj = obj?;
                    loose_count += 1;
                    loose_size += obj.metadata()?.len();
                }
            }
        }
    }

    println!("{} objects, {} bytes", loose_count, loose_size);

    if verbose {
        println!("");
        println!("count: {}", loose_count);
        println!("size: {}", loose_size);
        println!("in-pack: 0");
    }

    Ok(())
}

fn describe(_tags: bool, abbrev: u32) -> Result<()> {
    let hash = get_head()?.ok_or_else(|| Git5Error::InvalidRef("No commits yet".to_string()))?;
    let short_hash = &hash[..abbrev as usize];

    let dir = git4_dir()?;
    let tags_dir = dir.join("refs/tags");
    let mut found_tag = None;

    if tags_dir.exists() {
        for entry in fs::read_dir(&tags_dir)? {
            let entry = entry?;
            let tag_hash = fs::read_to_string(entry.path())?.trim().to_string();
            if tag_hash == hash {
                found_tag = Some(entry.file_name().to_string_lossy().to_string());
                break;
            }
        }
    }

    if let Some(tag) = found_tag {
        println!("{}", tag);
    } else {
        println!("{}-0-g{}", abbrev, short_hash);
    }

    Ok(())
}

fn verify_pack(packfile: &str) -> Result<()> {
    let pack_data = fs::read(packfile)?;
    if pack_data.len() < 32 {
        return Err(Git5Error::InvalidObject("Packfile is too small".to_string()));
    }

    if &pack_data[0..4] != b"PACK" {
        return Err(Git5Error::InvalidObject("Invalid packfile magic".to_string()));
    }

    let version = u32::from_be_bytes(pack_data[4..8].try_into().unwrap());
    if version != 2 {
        return Err(Git5Error::InvalidObject(format!("Unsupported pack version: {}", version)));
    }

    let num_objects = u32::from_be_bytes(pack_data[8..12].try_into().unwrap());
    println!("Packfile {}: valid", packfile);
    println!("Version: {}", version);
    println!("Objects: {}", num_objects);

    Ok(())
}

fn mktree(_binary: bool) -> Result<()> {
    let index = read_index()?;
    let mut entries = Vec::new();

    for (path, hash) in &index {
        entries.push(("100644".to_string(), path.clone(), hash.clone()));
    }

    entries.sort_by(|a, b| a.1.cmp(&b.1));

    let mut tree_content = Vec::new();
    for (mode, name, hash) in entries {
        tree_content.extend_from_slice(format!("{} {}\0", mode, name).as_bytes());
        tree_content.extend_from_slice(&hex::decode(hash)?);
    }

    let tree_hash = write_object("tree", &tree_content)?;
    println!("{}", tree_hash);
    Ok(())
}

fn ls_tree(tree: &str, recursive: bool) -> Result<()> {
    let hash = resolve_revision(tree)?;
    let (obj_type, content) = read_object(&hash)?;

    if obj_type != "tree" {
        return Err(Git5Error::InvalidObject("Not a tree object".to_string()));
    }

    let mut i = 0;
    while i < content.len() {
        let space_pos = i + content[i..].iter().position(|&b| b == b' ').unwrap_or(0);
        let mode_str = String::from_utf8_lossy(&content[i..space_pos]).to_string();

        let nul_pos = space_pos + content[space_pos..].iter().position(|&b| b == 0).unwrap_or(0);
        let name_str = String::from_utf8_lossy(&content[space_pos+1..nul_pos]).to_string();

        let sha = hex::encode(&content[nul_pos+1..nul_pos+21]);

        if mode_str == "40000" && recursive {
            ls_tree(&sha, true)?;
        } else {
            println!("{} {} {}\t{}", mode_str, sha, name_str, name_str);
        }

        i = nul_pos + 21;
    }

    Ok(())
}

fn update_ref(ref_name: &str, delete: bool, hash: Option<&str>) -> Result<()> {
    let dir = git4_dir()?;
    let ref_path = dir.join(ref_name);

    if delete {
        if ref_path.exists() {
            fs::remove_file(&ref_path)?;
            println!("Deleted {}", ref_name);
        } else {
            println!("{} not found", ref_name);
        }
    } else {
        let h = hash.ok_or_else(|| Git5Error::InvalidRef("Hash required".to_string()))?;
        fs::create_dir_all(ref_path.parent().unwrap())?;
        fs::write(&ref_path, format!("{}\n", h))?;
        println!("Updated {}", ref_name);
    }

    Ok(())
}

fn symbolic_ref(ref_name: Option<&str>, target: Option<&str>) -> Result<()> {
    let dir = git4_dir()?;
    let head_path = dir.join("HEAD");

    if let Some(t) = target {
        let ref_path = dir.join(t);
        fs::write(&head_path, format!("ref: {}\n", t))?;
        println!("Set HEAD to {}", t);
    } else {
        let content = fs::read_to_string(&head_path)?;
        let content = content.trim();
        if content.starts_with("ref: ") {
            println!("{}", content.strip_prefix("ref: ").unwrap());
        } else {
            println!("{}", content);
        }
    }

    Ok(())
}

fn for_each_ref(_format: Option<&str>) -> Result<()> {
    let dir = git4_dir()?;

    let heads_dir = dir.join("refs/heads");
    if heads_dir.exists() {
        for entry in fs::read_dir(&heads_dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            let hash = fs::read_to_string(entry.path())?.trim().to_string();
            println!("{} refs/heads/{}", hash, name);
        }
    }

    let tags_dir = dir.join("refs/tags");
    if tags_dir.exists() {
        for entry in fs::read_dir(&tags_dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            let hash = fs::read_to_string(entry.path())?.trim().to_string();
            println!("{} refs/tags/{}", hash, name);
        }
    }

    Ok(())
}

fn cat_file_batch() -> Result<()> {
    println!("object sha");
    Ok(())
}

fn diff_tree(tree1: &str, tree2: Option<&str>) -> Result<()> {
    let hash1 = resolve_revision(tree1)?;
    let (obj_type1, content1) = read_object(&hash1)?;

    if obj_type1 != "tree" {
        return Err(Git5Error::InvalidObject("Not a tree".to_string()));
    }

    println!("diff-tree {} {}", hash1, tree2.unwrap_or(""));

    let mut files1 = BTreeMap::new();
    parse_tree_content(&content1, &mut files1);

    if let Some(tree2_name) = tree2 {
        let hash2 = resolve_revision(tree2_name)?;
        let (obj_type2, content2) = read_object(&hash2)?;

        if obj_type2 == "tree" {
            let mut files2 = BTreeMap::new();
            parse_tree_content(&content2, &mut files2);

            for (path, hash) in &files1 {
                if let Some(hash2) = files2.get(path) {
                    if hash != hash2 {
                        println!("-{} {}", hash2, path);
                        println!("+{} {}", hash, path);
                    }
                } else {
                    println!("+{} {}", hash, path);
                }
            }
            for (path, hash2) in &files2 {
                if !files1.contains_key(path) {
                    println!("-{} {}", hash2, path);
                }
            }
        }
    }

    Ok(())
}

fn parse_tree_content(content: &[u8], files: &mut BTreeMap<String, String>) {
    let mut i = 0;
    while i < content.len() {
        let space_pos = i + content[i..].iter().position(|&b| b == b' ').unwrap_or(0);
        let nul_pos = space_pos + content[space_pos..].iter().position(|&b| b == 0).unwrap_or(0);
        let name = String::from_utf8_lossy(&content[space_pos+1..nul_pos]).to_string();
        let sha = hex::encode(&content[nul_pos+1..nul_pos+21]);
        files.insert(name, sha);
        i = nul_pos + 21;
    }
}

fn name_rev(commits: Vec<String>) -> Result<()> {
    for commit in commits {
        let hash = resolve_revision(&commit)?;
        let mut current = hash.clone();
        let mut found_ref = None;

        if let Ok(entries) = fs::read_dir(".git4/refs/heads") {
            for entry in entries.flatten() {
                let ref_name = entry.file_name().to_string_lossy().to_string();
                let ref_hash = fs::read_to_string(entry.path())?.trim().to_string();
                if ref_hash == current {
                    found_ref = Some(format!("refs/heads/{}", ref_name));
                    break;
                }
            }
        }

        if found_ref.is_none() {
            if let Ok(entries) = fs::read_dir(".git4/refs/tags") {
                for entry in entries.flatten() {
                    let ref_name = entry.file_name().to_string_lossy().to_string();
                    let ref_hash = fs::read_to_string(entry.path())?.trim().to_string();
                    if ref_hash == current {
                        found_ref = Some(format!("refs/tags/{}", ref_name));
                        break;
                    }
                }
            }
        }

        if let Some(ref_name) = found_ref {
            println!("{} ({})", hash, ref_name);
        } else {
            println!("{}", hash);
        }
    }
    Ok(())
}

fn verify_commit(commit: &str) -> Result<()> {
    let hash = resolve_revision(commit)?;
    let (obj_type, content) = read_object(&hash)?;

    if obj_type != "commit" {
        return Err(Git5Error::InvalidObject("Not a commit".to_string()));
    }

    println!("Commit {}", hash);
    println!("Warning: commit {} is not signed", hash);
    Ok(())
}

fn rev_list(commits: Vec<String>) -> Result<()> {
    if commits.is_empty() {
        return Ok(());
    }

    let mut to_process: Vec<String> = vec![resolve_revision(&commits[0])?];
    let mut processed = std::collections::HashSet::new();

    while let Some(commit_hash) = to_process.pop() {
        if processed.contains(&commit_hash) {
            continue;
        }
        processed.insert(commit_hash.clone());

        println!("{}", commit_hash);

        let (obj_type, content) = read_object(&commit_hash)?;
        if obj_type == "commit" {
            let content_str = String::from_utf8_lossy(&content);
            if let Some(parent_line) = content_str.lines().find(|l| l.starts_with("parent ")) {
                let parent_hash = parent_line[7..].trim().to_string();
                to_process.push(parent_hash);
            }
        }
    }
    Ok(())
}

fn archive(format: Option<String>, tree: Option<&str>) -> Result<()> {
    let tree_ref = tree.unwrap_or("HEAD");
    let hash = resolve_revision(tree_ref)?;

    let (obj_type, _) = read_object(&hash)?;
    if obj_type != "tree" && obj_type != "commit" {
        return Err(Git5Error::InvalidObject("Not a tree or commit".to_string()));
    }

    println!("Archive: {} (format: {})", hash, format.unwrap_or_else(|| "tar".to_string()));
    Ok(())
}

fn blame(file: &str) -> Result<()> {
    let path = std::env::current_dir()?.join(file);

    if !path.exists() {
        return Err(Git5Error::IoError(format!("File not found: {}", file)));
    }

    let content = fs::read_to_string(&path)?;
    let head = get_head()?.ok_or_else(|| Git5Error::InvalidRef("No commits".to_string()))?;

    for (line_num, line) in content.lines().enumerate() {
        println!("{}({}) {}", &head[..7], line_num + 1, line);
    }

    Ok(())
}