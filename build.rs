#![allow(unused_variables)]
// adapted from: https://github.com/LyonSyonII/dioxus-tailwindcss/blob/main/build.rs

use std::path::Path;

fn main() {
    // Install required packages
    let toolchain = install_packages();

    // Compile TailwindCSS .css file
    let mut child = std::process::Command::new(toolchain)
        .args([
            "tailwind",
            "-i",
            "src/index.css",
            "-c",
            "tailwind.config.js",
            "-o",
            "public/tailwind.css",
            "--minify",
        ])
        .env("NODE_ENV", "production")
        .spawn()
        .expect("Failed to compile tailwind");
    let status = child.wait().unwrap();
    if !status.success() {
        panic!("Failed to compile tailwind");
    }
}

/// Installs required packages and selects toolchain to use.
///
/// It will prioritize `yarn` over `npm`.
///
/// # Panic
/// Will panic if none of the toolchains is installed.
fn install_packages() -> &'static str {
    let npm = if_windows("npm.cmd", "npm");
    let npx = if_windows("npx.cmd", "npx");

    if Path::new("node_modules").exists() {
        return npx;
    }

    match std::process::Command::new(npm).arg("install").spawn() {
        Ok(mut child) => {
            let res = child.wait().unwrap();
            if !res.success() {
                panic!("ERROR: Npm installation is needed.");
            }
            npx
        }
        Err(e) => panic!("ERROR: Npm installation is needed.\n{e}"),
    }
}

const fn if_windows(windows: &'static str, unix: &'static str) -> &'static str {
    #[cfg(windows)]
    {
        windows
    }
    #[cfg(unix)]
    {
        unix
    }
}
