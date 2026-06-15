use std::io::Write as _;

pub fn forge() {
    // 1. Registramos formalmente los cfgs personalizados para eliminar los warnings del compilador
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
        std::env::var("CARGO_MANIFEST_DIR").expect("foundry: Falta CARGO_MANIFEST_DIR");
    let out_dir = std::env::var("OUT_DIR").expect("foundry: Falta la variable OUT_DIR");
    let final_data_dir = std::path::Path::new(&manifest_dir)
        .join("target")
        .join("foundry_data");

    println!(
        "cargo:rerun-if-changed={}",
        final_data_dir
            .to_str()
            .expect("foundry: Path de datos inválido")
    );

    let cache_ready = final_data_dir.exists()
        && std::fs::read_dir(&final_data_dir)
            .map(|mut d| d.next().is_some())
            .unwrap_or(false);

    if cache_ready {
        write_env_injection_file(&out_dir, &final_data_dir);
        println!("cargo:rustc-cfg=foundry_forged");
        return;
    }

    if !is_release && !forge_forced {
        write_env_injection_file(&out_dir, &final_data_dir);
        println!("cargo:rerun-if-changed=src/");
        return;
    }

    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=Cargo.lock");

    let capture_target_dir = std::path::Path::new(&out_dir).join("foundry_capture_target");
    std::fs::create_dir_all(&final_data_dir)
        .expect("foundry: No se pudo crear el directorio definitivo de datos");

    // 🛠️ CAPTURA AUTOMÁTICA NATIVA (Tu diseño original)
    let mut command = std::process::Command::new("cargo");
    command
        .args(&[
            "test",
            "--target-dir",
            capture_target_dir
                .to_str()
                .expect("foundry: Ruta temporal inválida"),
            "--",
            "__foundry_capture_for_",
        ])
        .current_dir(&manifest_dir)
        .env("FOUNDRY_CAPTURE_PASS", "1")
        .env("FOUNDRY_OUT_DIR_INJECT", final_data_dir.to_str().unwrap());

    command.env_remove("CARGO_MAKEFLAGS");
    command.env_remove("MFLAGS");
    command.env_remove("MAKEFLAGS");

    let status = command
        .status()
        .expect("foundry: Fallo crítico al ejecutar la captura automatizada");

    write_env_injection_file(&out_dir, &final_data_dir);

    if status.success() {
        println!("cargo:rustc-cfg=foundry_forged");
    } else {
        println!("cargo:warning=foundry: Fase de captura omitida o sin tests asociados.");
    }
}

fn write_env_injection_file(out_dir: &str, final_data_dir: &std::path::Path) {
    let env_file_path = std::path::Path::new(out_dir).join("foundry_env.rs");
    let mut file =
        std::fs::File::create(&env_file_path).expect("foundry: No se pudo crear foundry_env.rs");

    let mut map_branches = String::new();

    if final_data_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(final_data_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "matrix") {
                    if let Some(func_name) = path.file_stem().and_then(|s| s.to_str()) {
                        if let Ok(bytes) = std::fs::read(&path) {
                            let bytes_literal = format!("{:?}", bytes);
                            map_branches.push_str(&format!(
                                "\"{}\" => Some(&{}),\n",
                                func_name, bytes_literal
                            ));
                        }
                    }
                }
            }
        }
    }

    // Corregido el duplicado estructural del match fallback
    let code = format!(
        "#[allow(dead_code)]\n\
        pub fn get_matrix_bytes(name: &str) -> Option<&'static [u8]> {{\n\
            match name {{\n\
                {}\n\
                _ => None,\n\
            }}\n\
        }}",
        if map_branches.is_empty() {
            ""
        } else {
            &map_branches
        }
    );

    file.write_all(code.as_bytes())
        .expect("foundry: Error al escribir en foundry_env.rs");
}
