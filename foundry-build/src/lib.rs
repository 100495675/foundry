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

    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("foundry: Falta la variable CARGO_MANIFEST_DIR");
    let final_data_dir = std::path::Path::new(&manifest_dir)
        .join("target")
        .join("foundry_data");

    // 🔴 CRÍTICO 2: En sistemas multifunción, el script de build no puede comprobar un único archivo estático.
    // Comprobamos si el directorio de datos existe y tiene contenido previo.
    if final_data_dir.exists()
        && std::fs::read_dir(&final_data_dir)
            .map(|mut d| d.next().is_some())
            .unwrap_or(false)
    {
        println!("cargo:rustc-cfg=foundry_forged");
        return;
    }

    if !is_release && !forge_forced {
        println!("cargo:rerun-if-changed=src/");
        return;
    }

    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=Cargo.lock");

    let out_dir = std::env::var("OUT_DIR")
        .expect("foundry: Falta la variable OUT_DIR en el entorno de compilación");
    let capture_target_dir = std::path::Path::new(&out_dir).join("foundry_capture_target");

    // Cambiamos unwrap() por expect descriptivos
    std::fs::create_dir_all(&final_data_dir)
        .expect("foundry: No se pudo crear el directorio definitivo de datos en target/");

    let mut command = std::process::Command::new("cargo");
    command
        .args(&[
            "test",
            "--target-dir",
            capture_target_dir
                .to_str()
                .expect("foundry: Ruta de destino temporal inválida"),
            "--",
            "__foundry_capture_for_",
        ])
        .current_dir(&manifest_dir)
        .env("FOUNDRY_CAPTURE_PASS", "1")
        .env("FOUNDRY_OUT_DIR_INJECT", final_data_dir.to_str().unwrap())
        .env("RUSTFLAGS", "--cfg foundry_capture_mode");

    command.env_remove("CARGO_MAKEFLAGS");
    command.env_remove("MFLAGS");
    command.env_remove("MAKEFLAGS");

    let status = command
        .status()
        .expect("foundry: Fallo crítico al ejecutar el subproceso de captura automatizada");

    if status.success() {
        println!("cargo:rustc-cfg=foundry_forged");
    } else {
        println!(
            "cargo:warning=foundry: Fase de captura omitida o no se encontraron tests de layout asociados."
        );
    }
}
