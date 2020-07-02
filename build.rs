use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs, io};

const MIGRATION_ROOT_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/migrations");

/// Returns a list of subdirectories in the root migration directory
pub fn get_migration_dirs() -> Result<Vec<PathBuf>, io::Error> {
    let dirs = fs::read_dir(MIGRATION_ROOT_DIR)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.is_dir())
        .collect();

    Ok(dirs)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=migrations");

    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let migration_dirs = get_migration_dirs().expect("could not get migrations");

    let dest_path = Path::new(&out_dir).join("migrations.rs");
    let mut migrations = fs::File::create(dest_path).expect("could not create migrations.rs");

    fs::create_dir_all(Path::new(&out_dir).join("migrations"))?;

    write!(
        migrations,
        r#"
        use std::collections::BTreeMap;

        pub fn init() -> (BTreeMap<&'static str, &'static str>, BTreeMap<&'static str, &'static str>) {{
            let mut up = BTreeMap::new();
            let mut down = BTreeMap::new();
        "#
    )?;

    for path in migration_dirs {
        let migration_path = path.as_path();
        let migration_name = migration_path.file_name().unwrap().to_str().unwrap();

        println!(
            "cargo:rerun-if-changed=migrations/{}/up.sql",
            migration_name
        );
        println!(
            "cargo:rerun-if-changed=migrations/{}/down.sql",
            migration_name
        );

        write!(
            migrations,
            r#"
            up.insert("{migration_name}", include_str!("{migration_path}/up.sql"));
            down.insert("{migration_name}", include_str!("{migration_path}/down.sql"));
            "#,
            migration_path = migration_path
                .to_str()
                .expect("could not encode migration path as utf-8")
                .escape_default(),
            migration_name = migration_name.escape_default(),
        )?;
    }

    write!(
        migrations,
        r#"
            (up, down)
        }}
        "#
    )?;

    Ok(())
}
