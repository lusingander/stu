use std::process::Command;

// https://stackoverflow.com/questions/43753491/include-git-commit-hash-as-string-into-rust-program
fn main() {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    let hash = String::from_utf8(output.stdout).unwrap();
    let short_hash = &hash[..7];
    println!("cargo:rustc-env=BUILD_COMMIT_HASH={}", short_hash);
}
