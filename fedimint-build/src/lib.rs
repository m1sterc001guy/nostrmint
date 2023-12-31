use std::env;
use std::path::Path;
use std::process::Command;

/// Env variable to set to force git hash during build process
const FORCE_GIT_HASH_ENV: &str = "FEDIMINT_BUILD_FORCE_GIT_HASH";

/// Env variable the cargo will set during crate build to pass the detected git
/// hash to the binary itself.
const GIT_HASH_ENV: &str = "FEDIMINT_BUILD_CODE_VERSION";

fn set_code_version_inner() -> Result<(), String> {
    println!("cargo:rerun-if-env-changed={FORCE_GIT_HASH_ENV}");

    if let Ok(hash) = env::var(FORCE_GIT_HASH_ENV) {
        eprintln!("Forced hash via {FORCE_GIT_HASH_ENV} to {hash}");
        println!("cargo:rustc-env={GIT_HASH_ENV}={hash}");
        return Ok(());
    }
    // TODO: We're going to need some extra handling here for published crates being
    // built somewhere in the `$HOME/.cargo/...`, probably detecting it and
    // using a release version instead.

    // Note: best effort approach to force a re-run when the git hash in
    // the local repo changes without wrecking the incremental compilation
    // completely.
    for base in [
        // The relative path of git files might vary, so we just try a lot of cases.
        // If you go deeper than that, you're silly.
        ".",
        "..",
        "../..",
        "../../..",
        "../../../..",
        "../../../../..",
    ] {
        let p = &format!("{base}/.git/HEAD");
        if Path::new(&p).exists() {
            println!("cargo:rerun-if-changed={p}");
        }
        // Common(?) `git workdir` setup
        let p = &format!("{base}/HEAD");
        if Path::new(&p).exists() {
            println!("cargo:rerun-if-changed={p}");
        }
    }

    let output = match Command::new("git").args(["rev-parse", "HEAD"]).output() {
        Ok(output) => output,
        Err(e) => {
            return Err(format!("Failed to execute `git` command: {e}"));
        }
    };

    if !output.status.success() {
        return Err(format!(
            "`git` command failed: stderr: {}; stdout: {}",
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        ));
    }

    let hash = match String::from_utf8(output.stdout) {
        Ok(hash) => hash.trim().to_string(),
        Err(e) => {
            return Err(format!("Invalid UTF-8 sequence detected: {e}"));
        }
    };

    println!("cargo:rustc-env={GIT_HASH_ENV}={hash}");

    Ok(())
}

pub fn set_code_version() {
    match set_code_version_inner() {
        Ok(()) => {}
        Err(e) => {
            panic!("Failed to detect git hash version: {e}. Set {FORCE_GIT_HASH_ENV} to skip this check")
        }
    }
}
