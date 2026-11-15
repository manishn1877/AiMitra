// build.rs - llama.cpp native build (optional)
//
// The default build path does NOT build llama.cpp from source. The app runs
// llama.cpp's `llama-server` binary as a subprocess (see `find_llama_binary`
// in main.rs), so the native library is not needed for normal operation.
//
// To enable building llama.cpp from GitHub at compile time (requires git +
// cmake + network), pass the `--features llama-native` flag or set
// `CODE_MITRA_BUILD_LLAMA=1`.

use std::path::{Path, PathBuf};

fn main() {
    tauri_build::build();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo::rustc-check-cfg=cfg(mobile)");

    let build_llama = std::env::var("CODE_MITRA_BUILD_LLAMA")
        .map(|v| v == "1")
        .unwrap_or(false);

    if !build_llama {
        println!("cargo:warning=Skipping llama.cpp native build (default). Set CODE_MITRA_BUILD_LLAMA=1 to enable.");
        return;
    }

    build_llama_cpp();
}

fn build_llama_cpp() {
    use std::env;
    use std::process::Command;

    const LLAMA_CPP_REPO: &str = "https://github.com/ggerganov/llama.cpp";

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let llama_dir = out_dir.join("llama.cpp");

    // Clone or update llama.cpp
    if !llama_dir.exists() {
        println!("cargo:warning=Cloning llama.cpp from GitHub...");
        let _ = Command::new("git")
            .args(["config", "--global", "core.longpaths", "true"])
            .status();
        let status = Command::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                "--branch",
                "master",
                "-c",
                "core.longpaths=true",
                LLAMA_CPP_REPO,
                &llama_dir.to_string_lossy(),
            ])
            .status()
            .expect("Failed to clone llama.cpp. Is git installed?");

        if !status.success() {
            panic!("Failed to clone llama.cpp repository");
        }
    }

    // Build llama.cpp with CMake
    let build_dir = llama_dir.join("build");
    std::fs::create_dir_all(&build_dir).expect("Failed to create build directory");

    let mut cmake_args = vec![
        "-DCMAKE_BUILD_TYPE=Release".to_string(),
        "-DLLAMA_BUILD_SERVER=OFF".to_string(), // We use native library, not server
        "-DLLAMA_BUILD_EXAMPLES=OFF".to_string(),
        "-DLLAMA_BUILD_TESTS=OFF".to_string(),
        "-DLLAMA_BUILD_TOOLS=OFF".to_string(),
    ];

    #[cfg(target_os = "windows")]
    {
        cmake_args.push("-DLLAMA_CUBLAS=OFF".to_string());
        cmake_args.push("-DLLAMA_CLBLAST=OFF".to_string());
        cmake_args.push("-DLLAMA_BLAS=ON".to_string());
        cmake_args.push("-DLLAMA_BLAS_VENDOR=OpenBLAS".to_string());
    }

    #[cfg(target_os = "macos")]
    {
        cmake_args.push("-DLLAMA_METAL=ON".to_string());
        cmake_args.push("-DLLAMA_ACCELERATE=ON".to_string());
    }

    #[cfg(target_os = "linux")]
    {
        cmake_args.push("-DLLAMA_CUBLAS=OFF".to_string());
        cmake_args.push("-DLLAMA_CLBLAST=OFF".to_string());
        cmake_args.push("-DLLAMA_BLAS=ON".to_string());
        cmake_args.push("-DLLAMA_BLAS_VENDOR=OpenBLAS".to_string());
    }

    println!("cargo:warning=Configuring llama.cpp with CMake...");
    let status = Command::new("cmake")
        .args(&cmake_args)
        .arg(&llama_dir)
        .current_dir(&build_dir)
        .status()
        .expect("Failed to run cmake. Is CMake installed?");

    if !status.success() {
        panic!("CMake configuration failed");
    }

    println!("cargo:warning=Building llama.cpp (this may take a few minutes)...");
    let status = Command::new("cmake")
        .args(["--build", ".", "--config", "Release", "--parallel"])
        .current_dir(&build_dir)
        .status()
        .expect("Failed to build llama.cpp");

    if !status.success() {
        panic!("llama.cpp build failed");
    }

    // Find the built library
    let lib_name = if cfg!(target_os = "windows") {
        "llama.lib"
    } else if cfg!(target_os = "macos") {
        "libllama.dylib"
    } else {
        "libllama.a"
    };

    let lib_path = find_library(&build_dir, lib_name).expect("Built llama library not found");

    // Link the library
    println!(
        "cargo:rustc-link-search=native={}",
        lib_path.parent().unwrap().display()
    );
    println!("cargo:rustc-link-lib=static=llama");

    // Also link required system libraries
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-lib=ole32");
        println!("cargo:rustc-link-lib=ws2_32");
        println!("cargo:rustc-link-lib=openblas");
    }

    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=Metal");
        println!("cargo:rustc-link-lib=framework=Accelerate");
    }

    #[cfg(target_os = "linux")]
    {
        println!("cargo:rustc-link-lib=pthread");
        println!("cargo:rustc-link-lib=dl");
        println!("cargo:rustc-link-lib=m");
        println!("cargo:rustc-link-lib=openblas");
    }

    println!("cargo:warning=llama.cpp built successfully!");
    println!("cargo:warning=Library: {}", lib_path.display());
}

fn find_library(dir: &Path, name: &str) -> Option<PathBuf> {
    let candidates = vec![
        dir.join(name),
        dir.join("Release").join(name),
        dir.join("lib").join(name),
        dir.join("src").join(name),
    ];

    for candidate in candidates {
        if candidate.exists() {
            return Some(candidate);
        }
    }

    for entry in walkdir::WalkDir::new(dir) {
        let entry = entry.ok()?;
        if entry.file_name() == name {
            return Some(entry.path().to_path_buf());
        }
    }

    None
}
