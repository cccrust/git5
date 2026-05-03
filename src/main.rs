use clap::{Parser, Subcommand};
use git5::commands as cmd;

#[derive(Parser)]
#[command(name = "git5", about = "A lightweight git clone in Rust")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    HashObject {
        #[arg(short)]
        write: bool,
        file: String,
    },
    CatFile {
        #[arg(short = 'p')]
        print: bool,
        #[arg(short = 's')]
        size: bool,
        #[arg(short = 't')]
        type_flag: bool,
        object: String,
    },
    WriteTree,
    CommitTree {
        tree: String,
        #[arg(short)]
        parent: Option<String>,
        #[arg(short)]
        message: String,
    },
    Add {
        files: Vec<String>,
    },
    Commit {
        #[arg(short)]
        message: String,
    },
    Log,
    Branch {
        name: Option<String>,
    },
    Checkout {
        #[arg(short)]
        b: bool,
        name: String,
    },
    Status,
    Diff {
        file: String,
    },
    Merge {
        branch: String,
    },
    Clone {
        source: String,
        dest: String,
    },
    Push {
        remote_path: String,
        branch: String,
    },
    Fetch {
        remote_path: String,
    },
    Remote {
        action: String,
        name: String,
        url: String,
    },
    LsRemote {
        remote: String,
    },
    UnpackObjects {
        packfile: String,
    },
    Config {
        #[arg(long = "list")]
        list: bool,
        key: Option<String>,
        value: Option<String>,
    },
    Tag {
        #[arg(short)]
        delete: bool,
        name: Option<String>,
    },
    Rm {
        files: Vec<String>,
    },
    LsFiles {
        #[arg(long = "cached")]
        cached: bool,
    },
    RevParse {
        #[arg(short)]
        short: bool,
        #[arg(default_value = "HEAD")]
        revision: String,
    },
    ShowRef {
        #[arg(long = "heads")]
        heads: bool,
        #[arg(long = "tags")]
        tags: bool,
    },
    CountObjects {
        #[arg(long = "verbose")]
        verbose: bool,
    },
    Describe {
        #[arg(long = "tags")]
        tags: bool,
        #[arg(long = "abbrev", default_value = "7")]
        abbrev: u32,
    },
    VerifyPack {
        packfile: String,
    },
    MkTree {
        #[arg(short)]
        binary: bool,
    },
    LsTree {
        #[arg(short)]
        recursive: bool,
        tree: String,
    },
    UpdateRef {
        #[arg(short)]
        delete: bool,
        ref_name: String,
        hash: Option<String>,
    },
    SymbolicRef {
        ref_name: Option<String>,
        target: Option<String>,
    },
    ForEachRef {
        #[arg(long = "format")]
        format: Option<String>,
    },
    CatFileBatch {
        #[arg(short)]
        batch: bool,
    },
    DiffTree {
        tree1: String,
        tree2: Option<String>,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => cmd::run(cmd::Command::Init)?,
        Commands::HashObject { write, file } => cmd::run(cmd::Command::HashObject { write, file })?,
        Commands::CatFile { print, size, type_flag, object } => cmd::run(cmd::Command::CatFile { print, size, type_flag, object })?,
        Commands::WriteTree => cmd::run(cmd::Command::WriteTree)?,
        Commands::CommitTree { tree, parent, message } => cmd::run(cmd::Command::CommitTree { tree, parent, message })?,
        Commands::Add { files } => cmd::run(cmd::Command::Add { files })?,
        Commands::Commit { message } => cmd::run(cmd::Command::Commit { message })?,
        Commands::Log => cmd::run(cmd::Command::Log)?,
        Commands::Branch { name } => cmd::run(cmd::Command::Branch { name })?,
        Commands::Checkout { b, name } => cmd::run(cmd::Command::Checkout { create_branch: b, name })?,
        Commands::Status => cmd::run(cmd::Command::Status)?,
        Commands::Diff { file } => cmd::run(cmd::Command::Diff { file })?,
        Commands::Merge { branch } => cmd::run(cmd::Command::Merge { branch })?,
        Commands::Clone { source, dest } => cmd::run(cmd::Command::Clone { source, dest })?,
        Commands::Push { remote_path, branch } => cmd::run(cmd::Command::Push { remote_path, branch })?,
        Commands::Fetch { remote_path } => cmd::run(cmd::Command::Fetch { remote_path })?,
        Commands::Remote { action, name, url } => cmd::run(cmd::Command::Remote { action, name, url })?,
        Commands::LsRemote { remote } => cmd::run(cmd::Command::LsRemote { remote })?,
        Commands::UnpackObjects { packfile } => cmd::run(cmd::Command::UnpackObjects { packfile })?,
        Commands::Config { list, key, value } => cmd::run(cmd::Command::Config { list, key, value })?,
        Commands::Tag { delete, name } => cmd::run(cmd::Command::Tag { delete, name })?,
        Commands::Rm { files } => cmd::run(cmd::Command::Rm { files })?,
        Commands::LsFiles { cached } => cmd::run(cmd::Command::LsFiles { cached })?,
        Commands::RevParse { short, revision } => cmd::run(cmd::Command::RevParse { short, revision })?,
        Commands::ShowRef { heads, tags } => cmd::run(cmd::Command::ShowRef { heads, tags })?,
        Commands::CountObjects { verbose } => cmd::run(cmd::Command::CountObjects { verbose })?,
        Commands::Describe { tags, abbrev } => cmd::run(cmd::Command::Describe { tags, abbrev })?,
        Commands::VerifyPack { packfile } => cmd::run(cmd::Command::VerifyPack { packfile })?,
        Commands::MkTree { binary } => cmd::run(cmd::Command::MkTree { binary })?,
        Commands::LsTree { recursive, tree } => cmd::run(cmd::Command::LsTree { recursive, tree })?,
        Commands::UpdateRef { delete, ref_name, hash } => cmd::run(cmd::Command::UpdateRef { delete, ref_name, hash })?,
        Commands::SymbolicRef { ref_name, target } => cmd::run(cmd::Command::SymbolicRef { ref_name, target })?,
        Commands::ForEachRef { format } => cmd::run(cmd::Command::ForEachRef { format })?,
        Commands::CatFileBatch { batch } => cmd::run(cmd::Command::CatFileBatch { batch })?,
        Commands::DiffTree { tree1, tree2 } => cmd::run(cmd::Command::DiffTree { tree1, tree2 })?,
    }

    Ok(())
}