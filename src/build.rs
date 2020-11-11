use std::process::Command;

fn main() {
    let long = get_github_build(false);
    let short = get_github_build(true);
    println!("cargo:rustc-env=GIT_HASH_LONG={}", long);
    println!("cargo:rustc-env=GIT_HASH_SHORT={}", short);
}

pub fn get_github_build(short: bool) -> String {
    let mut args = vec!["rev-parse"];
    if short {
        args.push("--short");
    }

    args.push("HEAD");
    if let Ok(output) = Command::new("git").args(&args).output() {
        String::from_utf8(output.stdout).unwrap()
    } else {
        String::new()
    }
}
