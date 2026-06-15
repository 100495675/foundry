// foundry-build/src/lib.rs
use std::io::Write as _;

pub fn forge() {
    println!("cargo:rustc-check-cfg=cfg(foundry_forged)");
    println!("cargo:rerun-if-env-changed=FOUNDRY_FORGE");
    println!("cargo:rerun-if-env-changed=FOUNDRY_FORCE_REGEN");

    // Evitamos bucles recursivos infinitos cuando el Cargo hijo despierte
    if std::env::var("FOUNDRY_CAPTURE_PASS").is_ok() {
        return;
    }

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let pkg_name = std::env::var("CARGO_PKG_NAME").unwrap();

    let final_data_dir = std::path::Path::new(&manifest_dir)
        .join("target")
        .join("foundry_data");

    // Monitoreamos la carpeta de datos local de la aplicación
    println!(
        "cargo:rerun-if-changed={}",
        final_data_dir.to_str().unwrap()
    );

    // ⚡ BYPASS DE REGEN EN BUILD:
    // Comprobamos si ya existen archivos `.matrix` generados previamente.
    // Si la caché existe y NO se ha pedido una regeneración forzada, nos saltamos el test hijo.
    let mut tiene_cache = false;
    if final_data_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&final_data_dir) {
            tiene_cache = entries
                .flatten()
                .any(|e| e.path().extension().map_or(false, |ext| ext == "matrix"));
        }
    }

    let forzar_regen = std::env::var("FOUNDRY_FORCE_REGEN").is_ok();
    let mut hijo_exitoso = true;

    if !tiene_cache || forzar_regen {
        // Solo entramos aquí si es la primera compilación limpia o si pides reconstruir
        let capture_target_dir = std::path::Path::new(&out_dir).join("foundry_capture_target");
        std::fs::create_dir_all(&final_data_dir).unwrap();

        let mut command = std::process::Command::new("cargo");
        command
            .args(&[
                "test",
                "-p",
                &pkg_name,
                "--target-dir",
                capture_target_dir.to_str().unwrap(),
                "--",
                "__foundry_capture_for_",
            ])
            .current_dir(&manifest_dir)
            .env("FOUNDRY_CAPTURE_PASS", "1")
            .env("FOUNDRY_OUT_DIR_INJECT", final_data_dir.to_str().unwrap());

        command.env_remove("CARGO_MAKEFLAGS");
        command.env_remove("MFLAGS");
        command.env_remove("MAKEFLAGS");
        command.env_remove("CARGO_RECURSION_LIMIT");

        let status = command.status().unwrap_or_else(|_| {
            panic!("foundry: Imposible lanzar el subproceso de captura");
        });

        hijo_exitoso = status.success();

        if !hijo_exitoso {
            let _ = std::fs::remove_dir_all(&final_data_dir);
            std::fs::create_dir_all(&final_data_dir).unwrap();
        }
    }

    // Generación del archivo del mapa estático en el OUT_DIR
    let env_file_path = std::path::Path::new(&out_dir).join("foundry_env.rs");
    let mut file = std::fs::File::create(&env_file_path).unwrap();

    let mut map_branches = String::new();
    let mut cache_ready = false;

    if final_data_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&final_data_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "matrix") {
                    if let Some(func_name) = path.file_stem().and_then(|s| s.to_str()) {
                        if let Ok(bytes) = std::fs::read(&path) {
                            map_branches.push_str(&format!(
                                "\"{}\" => {{ \
                                    #[repr(align(8))]\
                                    struct AlignedData([u8; {}]);\
                                    static ALIGNED: AlignedData = AlignedData({:?});\
                                    Some(&ALIGNED.0)\
                                }},\n",
                                func_name,
                                bytes.len(),
                                bytes
                            ));
                            cache_ready = true;
                        }
                    }
                }
            }
        }
    }

    let code = format!(
        "#[allow(dead_code)]\n\
        pub fn get_matrix_bytes(name: &str) -> Option<&'static [u8]> {{\n\
            match name {{\n\
                {}\n\
                _ => None,\n\
            }}\n\
        }}",
        map_branches
    );

    file.write_all(code.as_bytes()).unwrap();

    // Activamos la optimización estática si tenemos datos válidos en el mapa
    if hijo_exitoso && cache_ready {
        println!("cargo:rustc-cfg=foundry_forged");
    }
}
