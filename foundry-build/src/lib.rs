pub fn forge() {
    println!("cargo:rustc-check-cfg=cfg(foundry_forged)");
    println!("cargo:rustc-check-cfg=cfg(foundry_capture_mode)");
    println!("cargo:rerun-if-env-changed=FOUNDRY_FORGE");

    if std::env::var("FOUNDRY_CAPTURE_PASS").is_ok() {
        return;
    }

    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let is_release = profile == "release";
    let forge_forced = std::env::var("FOUNDRY_FORGE")
        .map(|v| v == "1")
        .unwrap_or(false);

    if !is_release && !forge_forced {
        println!("cargo:rerun-if-changed=src/");
        return;
    }

    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=Cargo.lock");

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("Missing CARGO_MANIFEST_DIR");
    let out_dir = std::env::var("OUT_DIR").expect("Missing OUT_DIR");
    let capture_target_dir = std::path::Path::new(&out_dir).join("foundry_capture_target");

    let mut command = std::process::Command::new("cargo");
    command
        .args(&[
            "test",
            "--target-dir",
            capture_target_dir.to_str().unwrap(),
            "--",
            "__foundry_capture_for_",
        ])
        .current_dir(&manifest_dir)
        .current_dir(&manifest_dir)
        .env("FOUNDRY_CAPTURE_PASS", "1")
        .env("RUSTFLAGS", "--cfg foundry_capture_mode");

    command.env_remove("CARGO_MAKEFLAGS");
    command.env_remove("MFLAGS");
    command.env_remove("MAKEFLAGS");

    let status = command
        .status()
        .expect("Critical failure running foundry compiler capture step");

    if status.success() {
        let data_path = std::path::Path::new(&manifest_dir)
            .join("target")
            .join("foundry_data");

        println!("cargo:rerun-if-changed={}", data_path.to_str().unwrap());
        println!("cargo:rustc-cfg=foundry_forged");
    } else {
        println!(
            "cargo:warning=foundry: Capture phase skipped or found no automated layout tests."
        );
    }
}
