// foundry-build/src/lib.rs
use std::io::Write as _;

pub fn forge() {
    println!("cargo:rustc-check-cfg=cfg(foundry_forged)");

    // 1. Evitamos bucles recursivos infinitos cuando el Cargo hijo despierte
    if std::env::var("FOUNDRY_CAPTURE_PASS").is_ok() {
        return;
    }

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let pkg_name = std::env::var("CARGO_PKG_NAME").unwrap();

    // Rastreamos la carpeta de código de la aplicación
    let src_dir = std::path::Path::new(&manifest_dir).join("src");
    println!("cargo:rerun-if-changed={}", src_dir.to_str().unwrap());

    let final_data_dir = std::path::Path::new(&manifest_dir)
        .join("target")
        .join("foundry_data");

    println!(
        "cargo:rerun-if-changed={}",
        final_data_dir.to_str().unwrap()
    );

    // 2. DETECTOR DE MODIFICACIONES POR HARDWARE (MARCAS DE TIEMPO)
    // Buscamos la fecha de modificación más reciente de cualquier archivo de código fuente
    let mut ultima_modificacion_src = std::time::SystemTime::UNIX_EPOCH;
    if let Ok(entries) = std::fs::read_dir(&src_dir) {
        for entry in entries.flatten() {
            if entry.path().extension().map_or(false, |ext| ext == "rs") {
                if let Ok(meta) = entry.metadata() {
                    if let Ok(mod_time) = meta.modified() {
                        if mod_time > ultima_modificacion_src {
                            ultima_modificacion_src = mod_time;
                        }
                    }
                }
            }
        }
    }

    // Buscamos la fecha de modificación del archivo .matrix más viejo en el disco
    let mut tiene_cache = false;
    let mut ultima_modificacion_cache = std::time::SystemTime::now();

    if final_data_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&final_data_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "matrix") {
                    tiene_cache = true;
                    if let Ok(meta) = entry.metadata() {
                        if let Ok(mod_time) = meta.modified() {
                            if mod_time < ultima_modificacion_cache {
                                ultima_modificacion_cache = mod_time;
                            }
                        }
                    }
                }
            }
        }
    }

    // 🔄 INVALIDACIÓN AUTOMÁTICA COHERENTE:
    // Si la última edición de tu código fuente es MÁS NUEVA que la caché guardada,
    // significa que el usuario ha cambiado un número o una lógica. Forzamos la regeneración.
    let codigo_ha_cambiado = ultima_modificacion_src > ultima_modificacion_cache;
    let mut hijo_exitoso = true;

    if !tiene_cache || codigo_ha_cambiado {
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

    // 3. INYECCIÓN DEL MAPA ALINEADO A 8 BYTES
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

    if hijo_exitoso && cache_ready {
        println!("cargo:rustc-cfg=foundry_forged");
    }
}
