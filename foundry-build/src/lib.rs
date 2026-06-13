pub fn forge() {
    println!("cargo:rustc-check-cfg=cfg(foundry_forged)");
    println!("cargo:rustc-check-cfg=cfg(foundry_capture_mode)");
    println!("cargo:rerun-if-env-changed=FOUNDRY_FORGE");

    if std::env::var("FOUNDRY_CAPTURE_PASS").is_ok() {
        return;
    }

    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let es_release = profile == "release";

    let forge_forzado = std::env::var("FOUNDRY_FORGE")
        .map(|v| v == "1")
        .unwrap_or(false);

    if !es_release && !forge_forzado {
        println!("cargo:rerun-if-changed=src/");
        return;
    }

    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=Cargo.lock");

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("Falta CARGO_MANIFEST_DIR");
    let out_dir = std::env::var("OUT_DIR").expect("Falta OUT_DIR");

    // Directorio completamente aislado fuera del árbol estándar para evitar colisiones de Locks
    let capture_target_dir = std::path::Path::new(&out_dir).join("foundry_capture_target");

    // --- PURGA DE ENTORNO DE CARGO ---
    // Eliminamos las variables con las que el Cargo padre controla al Cargo hijo.
    // Al limpiar esto, el subproceso se cree que es un comando independiente en una terminal limpia.
    let mut comando = std::process::Command::new("cargo");
    comando
        .args(&[
            "test",
            "--target-dir",
            capture_target_dir.to_str().unwrap(),
            "--",
            "__foundry_capture_for_",
        ])
        .current_dir(&manifest_dir)
        .env("FOUNDRY_CAPTURE_PASS", "1")
        .env("RUSTFLAGS", "--cfg foundry_capture_mode");

    // Purgamos banderas de concurrencia de Cargo para evitar que el hijo espere al padre
    comando.env_remove("CARGO_MAKEFLAGS");
    comando.env_remove("MFLAGS");
    comando.env_remove("MAKEFLAGS");

    let status = comando
        .status()
        .expect("Error crítico en la captura de foundry");

    if status.success() {
        let ruta_datos = std::path::Path::new(&manifest_dir)
            .join("target")
            .join("foundry_data");

        println!("cargo:rerun-if-changed={}", ruta_datos.to_str().unwrap());
        println!("cargo:rustc-cfg=foundry_foged");
    } else {
        println!("cargo:warning=foundry: la fase de captura falló o no encontró tests.");
    }
}
