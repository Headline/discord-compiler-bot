use std::env;
use std::process::Command;

fn main() {
    // Docker builds don't have git (or .git) available, so allow the hash
    // to be passed in through the environment instead.
    println!("cargo:rerun-if-env-changed=GIT_COMMIT_SHA");
    if std::path::Path::new(".git/HEAD").exists() {
        println!("cargo:rerun-if-changed=.git/HEAD");
    }

    let long = get_commit_hash();
    let short: String = long.chars().take(7).collect();
    println!("cargo:rustc-env=GIT_HASH_LONG={}", long);
    println!("cargo:rustc-env=GIT_HASH_SHORT={}", short);
}

fn get_commit_hash() -> String {
    if let Ok(hash) = env::var("GIT_COMMIT_SHA") {
        let hash = hash.trim();
        if !hash.is_empty() {
            return hash.to_string();
        }
    }

    match Command::new("git").args(["rev-parse", "HEAD"]).output() {
        Ok(output) if output.status.success() => String::from_utf8(output.stdout)
            .unwrap_or_default()
            .trim()
            .to_string(),
        _ => String::new(),
    }
}
