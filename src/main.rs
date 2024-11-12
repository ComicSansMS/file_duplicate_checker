use clap::{self, Parser};
use colored::Colorize;
use sha_256;
use std::{collections::HashMap, io::Write};

fn scan_on_directory(path: &std::path::Path) -> Result<HashMap<Hash, FileInfo>, std::io::Error> {
    let mut file_map = HashMap::new();
    scan_rec(path, &mut file_map)?;
    Ok(file_map)
}

fn scan_rec(
    path: &std::path::Path,
    filemap: &mut HashMap<Hash, FileInfo>,
) -> Result<(), std::io::Error> {
    let reader = std::fs::read_dir(path)?;
    for it in reader {
        let entry = it?;
        let path = entry.path();
        if path.is_dir() {
            scan_rec(&path, filemap)?;
        } else if path.is_file() {
            let (hash, size) = hash_file(&path)?;
            if filemap.contains_key(&hash) {
                filemap.get_mut(&hash).unwrap().add_path(path);
            } else {
                filemap.insert(hash, FileInfo::new(path, size));
            }
        }
    }
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
struct FileInfo {
    paths: Vec<std::path::PathBuf>,
    size: usize,
}

impl FileInfo {
    fn new(path: std::path::PathBuf, size: usize) -> Self {
        Self {
            paths: Vec::from(&[path]),
            size,
        }
    }

    fn add_path(self: &mut Self, path: std::path::PathBuf) {
        self.paths.push(path);
    }
}

fn hash_file(path: &std::path::Path) -> Result<(Hash, usize), std::io::Error> {
    let mut hasher = sha_256::Sha256::new();
    let data = std::fs::read(path)?;
    let hash = hasher.digest(&data);
    Ok((Hash::new(&hash), data.len()))
}

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct Hash {
    hash: [u8; 32],
}

impl Hash {
    fn new(h: &[u8; 32]) -> Self {
        Self { hash: *h }
    }
}

impl std::fmt::Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        for i in 0..32 {
            write!(f, "{:02x}", self.hash[i])?;
        }
        Ok(())
    }
}

fn handle_duplicates(
    file_map: HashMap<Hash, FileInfo>,
    do_fix: bool,
) -> Result<(), std::io::Error> {
    for (k, v) in file_map.iter().filter(|(_, v)| v.paths.len() > 1) {
        println!("Hash set {} (filesize: {} bytes):", k, v.size);
        for (idx, f) in v.paths.iter().enumerate() {
            println!(" {} - {:?}", idx + 1, f);
        }
        if do_fix {
            fix_duplicates(v)?;
        }
    }
    Ok(())
}

fn fix_duplicates(v: &FileInfo) -> Result<(), std::io::Error> {
    if let Some(index_to_keep) = loop {
        print!("Select one to keep (0 to keep all): ");
        std::io::stdout().flush()?;
        let mut str_index_to_keep = String::new();
        std::io::stdin().read_line(&mut str_index_to_keep)?;
        if let Ok(candidate_index_to_keep) = str_index_to_keep.trim().parse::<usize>() {
            if candidate_index_to_keep <= v.paths.len() {
                break if candidate_index_to_keep == 0 {
                    None
                } else {
                    Some(candidate_index_to_keep - 1)
                };
            } else {
                println!("Invalid index.")
            }
        } else {
            println!("Invalid input.");
        }
    } {
        for (idx, f) in v.paths.iter().enumerate() {
            if idx != index_to_keep {
                println!(" {} {:?}", "Deleting".red(), f);
                if let Err(e) = std::fs::remove_file(f) {
                    eprintln!("Unable to remove file: {}", e);
                }
            }
        }
    }
    Ok(())
}

fn run(target_dir: &std::path::Path, do_fix: bool) -> Result<(), std::io::Error> {
    handle_duplicates(scan_on_directory(target_dir)?, do_fix)
}

#[derive(clap::Parser, Debug)]
#[command(version, about)]
struct Cli {
    /// Path of the directory to scan
    target_path: std::path::PathBuf,
    /// Fix duplicates by selecting one file to keep
    #[arg(short = 'f', long)]
    do_fix: bool,
}

fn main() -> std::process::ExitCode {
    let cli = Cli::parse();
    let target_dir = cli.target_path;
    println!("Scanning directory {target_dir:?} for duplicates...");
    match run(&target_dir, cli.do_fix) {
        Ok(_) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            std::eprintln!("Error while scanning: {e}");
            std::process::ExitCode::FAILURE
        }
    }
}
