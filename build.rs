use std::process::Command;

fn main() {
    // Inject git commit hash into build for version reporting
    let output = Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output();
    
    let git_hash = match output {
        Ok(output) if output.status.success() => {
            String::from_utf8(output.stdout).unwrap_or_else(|_| "unknown".to_string())
        }
        _ => "unknown".to_string(),
    };
    
    println!("cargo:rustc-env=GIT_HASH={}", git_hash.trim());
    println!("cargo:rerun-if-changed=.git/HEAD");
}