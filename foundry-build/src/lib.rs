pub fn forge() {
    println!("cargo:rustc-check-cfg=cfg(foundry_baked)");
    println!("cargo:rerun-if-env-changed=FOUNDRY_BAKE");

    if std::env::var("FOUNDRY_CAPTURE_PASS").is_ok() {
        return;
    }

    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let es_release = profile == "release";

    let bake_forzado = std::env::var("FOUNDRY_BAKE")
        .map(|v| v == "1")
        .unwrap_or(false);

    if !es_release && !bake_forzado {
        println!("cargo:rerun-if-changed=src/");
        return;
    }

    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=Cargo.lock");

    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("Falta CARGO_MANIFEST_DIR");
    let out_dir =
        std::env::var("OUT_DIR").expect("Falta OUT_DIR");

    let capture_target_dir = std::path::Path::new(&out_dir)
        .join("foundry_capture_target");

    let status = std::process::Command::new("cargo")
        .args(&[
            "test",
            "--features",
            "foundry-capture",
            "--target-dir",
            capture_target_dir.to_str().unwrap(),
            "--",
            "__foundry_capture_for_",
        ])
        .current_dir(&manifest_dir)
        .env("FOUNDRY_CAPTURE_PASS", "1")
        // Eliminamos la inyección del env OUT_DIR para que el test lea su propio CARGO_MANIFEST_DIR nativo
        .status()
        .expect("Error crítico en la captura de foundry");

    if status.success() {
        println!("cargo:rustc-cfg=foundry_baked");
    } else {
        println!("cargo:warning=foundry: la fase de captura falló.");
    }
}