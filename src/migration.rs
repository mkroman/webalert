use lazy_static::lazy_static;

use std::path::PathBuf;
use std::{env, fs, io};

// TODO: Add embedded migrations
lazy_static! {
    pub static ref MIGRATION_ROOT_DIR: PathBuf = env::current_dir()
        .map(|x| x.join("migrationslol"))
        .expect("could not get current working directory");
}

/// Returns an unsorted list of paths for each file in `./migrations/*`
fn get_migration_file_paths() -> Result<Vec<PathBuf>, io::Error> {
    let file_paths = fs::read_dir(MIGRATION_ROOT_DIR.as_path())?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.is_file())
        .collect();

    Ok(file_paths)
}

/// Returns list of migrations sorted by file name
pub fn get_migrations_sorted() -> Result<Vec<String>, io::Error> {
    let file_paths = get_migration_file_paths()?;

    let mut file_names: Vec<&str> = file_paths
        .iter()
        .filter_map(|x| x.file_name().and_then(|s| s.to_str()))
        .collect();

    file_names.sort();

    Ok(file_names.iter().map(|s| s.to_string()).collect())
}
