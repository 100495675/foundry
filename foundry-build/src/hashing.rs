//! Motor de cálculo de contexto semántico (AST + Lockfile).
//!
//! Proporciona utilidades para computar las firmas genéticas que identifican
//! de forma unívoca el estado del código fuente y las dependencias del proyecto.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;

/// Calcula el hash semántico del AST a partir de la representación textual del código fuente.
///
/// Este hash se utiliza como firma sintáctica que identifica de forma unívoca
/// el cuerpo de una función patrón.
pub fn compute_ast_hash(source: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);
    hasher.finish()
}

/// Calcula el hash de dependencias a partir del `Cargo.lock` del workspace.
///
/// Retorna `None` si el archivo no puede ser leído (proyecto sin lockfile).
pub fn compute_dependency_hash(lockfile_path: &Path) -> Option<u64> {
    let content = std::fs::read_to_string(lockfile_path).ok()?;
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    Some(hasher.finish())
}

/// Calcula un hash combinado del entorno de compilación actual.
///
/// Incorpora variables de entorno de Cargo y, si está disponible, el contenido
/// del `Cargo.lock` para producir una firma que capture tanto el estado del
/// código como el del ecosistema de dependencias.
pub fn compute_environment_hash() -> u64 {
    let mut hasher = DefaultHasher::new();

    // Incorporar información del entorno de compilación
    std::env::var("CARGO_PKG_VERSION")
        .unwrap_or_default()
        .hash(&mut hasher);
    std::env::var("CARGO_PKG_NAME")
        .unwrap_or_default()
        .hash(&mut hasher);
    std::env::var("TARGET")
        .unwrap_or_default()
        .hash(&mut hasher);
    std::env::var("PROFILE")
        .unwrap_or_default()
        .hash(&mut hasher);
    std::env::var("OPT_LEVEL")
        .unwrap_or_default()
        .hash(&mut hasher);
    std::env::var("DEBUG").unwrap_or_default().hash(&mut hasher);

    // Intentar incorporar el Cargo.lock
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let lock_path = Path::new(&manifest_dir).join("Cargo.lock");
        if let Some(dep_hash) = compute_dependency_hash(&lock_path) {
            dep_hash.hash(&mut hasher);
        }
    }

    hasher.finish()
}
