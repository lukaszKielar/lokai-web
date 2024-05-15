use std::io::{self, Write};
use std::process::Command;

fn main() {
    let output = Command::new("pnpm")
        .arg("build")
        .output()
        .expect("Cannot compile tailwind.css");
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();
    println!("cargo:rerun-if-changed=tailwind.css");
}
