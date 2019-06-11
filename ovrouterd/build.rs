use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    let commit = Command::new("git")
        .args(vec!("git", "rev-parse", "HEAD"))
        .output()
        .expect("Unable to get git commit date");
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    fs::write(out_dir.join("git-commit-date.txt"), commit_date());
}

fn commit_date() -> String {
    let output = Command::new("git")
        .args(&["log", "-1", "--date=short", "--pretty=format:%cd"])
        .output()
        .expect("Unable to get git commit date");
    ::std::str::from_utf8(&output.stdout)
        .unwrap()
        .trim()
        .to_owned()
}