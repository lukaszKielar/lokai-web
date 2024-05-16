use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=assets");
    println!("cargo:rerun-if-changed=templates");
    println!("cargo:rerun-if-changed=migrations");

    std::fs::remove_dir_all("static").unwrap_or_default();

    Command::new("tailwindcss")
        .args([
            "-c",
            "tailwind.config.js",
            "-i",
            "assets/styles/index.css",
            "-o",
            "static/index.css",
            "--minify",
        ])
        .status()
        .expect("failed to run tailwindcss");

    copy_files("public");
}

fn copy_files(dir: &str) {
    for entry in std::fs::read_dir(dir).expect("failed to read dir `public`") {
        let entry = entry.expect("failed to read entry");

        if entry.file_type().unwrap().is_dir() {
            copy_files(entry.path().to_str().unwrap());
        } else {
            let path = entry.path();
            let filename = path.file_name().unwrap().to_str().unwrap();
            let dest = format!("static/{}", filename);

            std::fs::copy(path, dest).expect("failed to copy file");
        }
    }
}
