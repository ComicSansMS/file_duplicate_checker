use sha_256;
use std::collections::HashMap;

fn scan_on_directory(path: std::path::PathBuf) -> Result<(), std::io::Error> {
    let mut file_map = HashMap::new();
    scan_rec(&path, &mut file_map)?;
    for (k, v) in file_map.iter().filter(|(_, v)| v.len() > 1) {
        println!("Hash set {:?}:", k);
        for f in v {
            println!(" - {:?}", f);
        }
    }
    Ok(())
}

fn scan_rec(
    path: &std::path::Path,
    filemap: &mut HashMap<[u8; 32], Vec<std::path::PathBuf>>,
) -> Result<(), std::io::Error> {
    let reader = std::fs::read_dir(path)?;
    for it in reader {
        let entry = it?;
        let path = entry.path();
        if path.is_dir() {
            scan_rec(&path, filemap)?;
        } else if path.is_file() {
            let hash = hash_file(&path)?;
            if filemap.contains_key(&hash) {
                filemap.get_mut(&hash).unwrap().push(path);
            } else {
                filemap.insert(hash, Vec::from([path]));
            }
        }
    }
    Ok(())
}

fn hash_file(path: &std::path::Path) -> Result<[u8; 32], std::io::Error> {
    let mut hasher = sha_256::Sha256::new();
    let data = std::fs::read(path)?;
    let hash = hasher.digest(&data);
    Ok(hash)
}

fn main() -> std::process::ExitCode {
    if std::env::args().len() < 2 {
        std::println!("Usage: {} <target_path>", std::env::args().nth(0).unwrap());
        return std::process::ExitCode::SUCCESS;
    }
    let target_dir = std::env::args().nth(1).unwrap();
    println!("Scanning directory {target_dir} for duplicates...");
    match scan_on_directory(std::path::PathBuf::from(target_dir)) {
        Ok(_) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            std::eprintln!("Error while scanning: {e}");
            std::process::ExitCode::FAILURE
        }
    }
}
