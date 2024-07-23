use serde::Serialize;
use std::collections::VecDeque;
use std::env;
use std::fs::{self, File};
// use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Serialize)]
struct FolderWeight {
    child: String,
    folder: String,
    weight: u64,
    human: String,
}

#[derive(Serialize)]
struct Data {
    total: u64,
    human: String,
    details: Vec<FolderWeight>,
}

fn resolve_path(relative_path: &str) -> PathBuf {
    let current_dir = env::current_dir().expect("Failed to get current directory");
    current_dir.join(relative_path)
}

fn get_abs_path(root: &Path, path: &Path) -> PathBuf {
    let full_path = root.join(path);

    if !full_path.exists() {
        eprintln!("Path does not exist: {:?}", full_path);
        std::process::exit(1);
    }

    match full_path.canonicalize() {
        Ok(canonical_path) => canonical_path,
        Err(e) => {
            eprintln!("Error canonicalizing path: {:?}", e);
            std::process::exit(1);
        }
    }
}

fn get_dir_size(path: &Path) -> u64 {
    let mut total = 0;
    let mut queue = VecDeque::new();
    queue.push_back(path.to_path_buf());

    while let Some(current_path) = queue.pop_front() {
        if let Ok(entries) = fs::read_dir(&current_path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let metadata = entry.metadata();
                    if metadata.is_err() {
                        continue;
                    }
                    let metadata = metadata.unwrap();
                    if metadata.is_file() {
                        total += metadata.len();
                    } else if metadata.is_dir() {
                        queue.push_back(entry.path());
                    }
                }
            }
        }
    }

    total
}

fn get_folders() -> Vec<String> {
    if let Ok(dependencies) = env::var("RC_DEPENDENCIES") {
        dependencies
            .split(',')
            .map(|s| s.trim().to_string())
            .collect()
    } else {
        vec!["node_modules", ".venv", "venv", ".git"]
            .iter()
            .map(|&s| s.to_string())
            .collect()
    }
}

fn sizeof_fmt(num: u64) -> String {
    let units = ["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB", "ZiB"];
    let mut num = num as f64;
    let mut unit = "YiB";

    for &u in &units {
        if num < 1024.0 {
            unit = u;
            break;
        }
        num /= 1024.0;
    }

    format!("{:.1} {}", num, unit)
}

fn scan(
    current_folder: &Path,
    absolute_parent_path: &Path,
    folders: &[&str],
    all_weights: &mut Vec<FolderWeight>,
) {
    if let Ok(entries) = fs::read_dir(current_folder) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                let file_name = entry.file_name();
                let file_name_str = file_name.to_str().unwrap_or_default();

                if path.is_symlink() || path.is_file() {
                    continue;
                }

                if folders.contains(&file_name_str) {
                    let full_child_path = get_abs_path(absolute_parent_path, &path);
                    let size = get_dir_size(&full_child_path);
                    all_weights.push(FolderWeight {
                        child: file_name_str.to_string(),
                        folder: full_child_path.to_string_lossy().to_string(),
                        weight: size,
                        human: sizeof_fmt(size),
                    });
                } else if path.is_dir() {
                    scan(&path, absolute_parent_path, folders, all_weights);
                }
            }
        }
    }
}

fn main() {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    // let folders = ["node_modules", ".venv", "venv", ".git"];

    let folders = get_folders();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("rc_dependencies v{} by Richard Carlier", VERSION);
        println!("");
        println!("Find dependencies folders and their sizes");
        println!("    {:?}", folders);
        println!("And export to a json file (or print to terminal)");
        println!("You can change folders name in .bash_profile (or equiv) via");
        println!("    export rc_dependencies=\"node_modules,.venv,venv,.git\" ");
        println!("And (or equiv)");
        println!("    source ~/.bash_profile");
        println!("");
        println!("Usage: {} <root_folder> [json_file]", args[0]);
        return;
    }

    let directory = &args[1];
    let resolved_path = resolve_path(directory);
    if !resolved_path.exists() {
        eprintln!("Error: Directory does not exist: {:?}", resolved_path);
        std::process::exit(1);
    }

    // root doit être &str  et resolved_path est
    // let root  = resolved_path;
    let root: &Path = resolved_path.as_path();

    let file_save = if args.len() > 2 { Some(&args[2]) } else { None };

    let folders_slice: Vec<&str> = folders.iter().map(|s| s.as_str()).collect();

    let mut all_weights = Vec::new();

    scan(root, root, &folders_slice, &mut all_weights);

    let total: u64 = all_weights.iter().map(|item| item.weight).sum();

    let data = Data {
        total,
        human: sizeof_fmt(total),
        details: all_weights,
    };

    if let Some(file_path) = file_save {
        let file = File::create(file_path).unwrap();
        serde_json::to_writer_pretty(file, &data).unwrap();
        println!("Data saved to {}", file_path);
    } else {
        println!("{}", serde_json::to_string_pretty(&data).unwrap());
    }
}
