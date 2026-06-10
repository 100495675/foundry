//! # foundry-build
//!
//! Utilidades del Script de Compilación para la Fragua Adaptativa.

pub mod capture;
pub mod hashing;

pub use capture::{examinar_estado_matriz, execute_matrix_capture, MatrixStatus};
pub use hashing::{compute_ast_hash, compute_dependency_hash, compute_environment_hash};

/// Función orquestadora maestra ejecutada por el build.rs del cliente.
pub fn forge() {
    // 1. Evitar bucles infinitos de reentrada en la subcompilación de captura
    if std::env::var("FOUNDRY_CAPTURE_PASS").is_ok() {
        return;
    }

    // --- EL FILTRO DE LIBERTAD: CONTROL DE runtime EN COMPILACIÓN ---
    // Si el usuario NO ha configurado la variable FOUNDRY_BAKE="1", el script
    // no hace absolutamente nada. El código corre en modo dinámico tradicional.
    if std::env::var("FOUNDRY_BAKE")
        .map(|v| v == "1")
        .unwrap_or(false)
    {
        let out_dir = std::env::var("OUT_DIR").expect("Falta la variable OUT_DIR");
        let manifest_dir =
            std::env::var("CARGO_MANIFEST_DIR").expect("Falta la variable CARGO_MANIFEST_DIR");

        // Informar a Cargo cuándo reevaluar el script si estamos horneando
        println!("cargo:rerun-if-changed=src/main.rs");
        println!("cargo:rerun-if-changed=Cargo.lock");

        let capture_target_dir = std::path::Path::new(&out_dir).join("foundry_capture_target");

        // Ejecutamos la suite de tests filtrando por las funciones instrumentales
        let status = std::process::Command::new("cargo")
            .args(&[
                "test",
                "--features",
                "foundry-capture",
                "--target-dir",
                capture_target_dir.to_str().unwrap(),
                "--",
                "__foundry_capture_",
            ])
            .current_dir(&manifest_dir)
            .env("FOUNDRY_CAPTURE_PASS", "1")
            .env("OUT_DIR", &out_dir)
            .status()
            .expect("Error crítico en la captura nativa de Foundry");

        if status.success() {
            // El horneado físico se ha completado: activamos la inyección estática en las macros
            println!("cargo:rustc-cfg=foundry_baked");
        } else {
            println!("cargo:warning=foundry: La fase de captura molecular falló.");
        }
    } else {
        // Modo Desarrollo Libre: Informamos a Cargo que reevalúe siempre para permitir iteraciones rápidas en caliente
        println!("cargo:rerun-if-changed=src/main.rs");
    }
}
