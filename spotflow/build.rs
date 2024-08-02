use sqlx::{self, Connection, SqliteConnection};
use std::{
    env,
    fs::{remove_file, File, OpenOptions},
    io::Write,
    path::Path,
};

fn main() {
    if std::env::var("DOCS_RS").is_ok() {
        return;
    }

    println!("cargo:rerun-if-changed=db_init.sql");
    println!("cargo:rerun-if-changed=.env");
    println!("cargo:rerun-if-changed=sqlx-data.json");

    let manifest_dir = env::vars()
        .find(|x| x.0 == "CARGO_MANIFEST_DIR")
        .expect("No out dir specified in build script")
        .1;
    let manifest_dir = Path::new(&manifest_dir);

    if env::var("CI").is_ok() {
        // We do not want to use sqlx in CI. Instead sqlx-data.json must be present.
        let sqlx_data = manifest_dir.join("sqlx-data.json");
        if !Path::exists(&sqlx_data) {
            panic!("When run in CI `sqlx-data.json` must exist. Use `cargo sqlx prepare` to create it and check it inside git.");
        }
        return;
    }

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let db_path = manifest_dir.join("dev.db");

    if db_path.exists() {
        remove_file(&db_path).unwrap();
    }
    File::create(&db_path).unwrap();

    OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&db_path)
        .expect("Unable to create or open db file");
    File::create(&db_path).unwrap();

    let db_path = db_path.to_string_lossy();

    rt.block_on(async {
        let mut conn = SqliteConnection::connect(&db_path).await.unwrap();
        sqlx::query(include_str!("./db_init.sql"))
            .execute(&mut conn)
            .await
            .unwrap();
    });

    let env_path = manifest_dir.join(".env");

    if !Path::new(&env_path).exists() {
        File::create(&env_path).unwrap();
    }

    let mut env_file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&env_path)
        .expect("Unable to open .env file");

    let env = format!("DATABASE_URL='sqlite:{db_path}'\n");
    env_file
        .write_all(env.as_bytes())
        .expect("Unable to write to `.env`");
}
