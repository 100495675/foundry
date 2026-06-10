use bincode::Options;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

/// Estado de vigencia de un artefacto `.matrix`.
#[derive(Debug)]
pub enum MatrixStatus {
    /// El artefacto es válido y coincide con el código actual.
    Valid,
    /// El artefacto no existe.
    Missing,
    /// El artefacto está desactualizado (código o dependencias cambiaron).
    Obsolete(String),
    /// El artefacto está corrupto (cabecera inválida).
    Corrupted,
}

/// Tamaño fijo de la cabecera `.matrix` en bytes.
const HEADER_SIZE: usize = 31;

/// Inspecciona atómicamente la matriz para certificar su correspondencia con el código actual.
///
/// Lee directamente la cabecera física del artefacto `.matrix` y compara las firmas
/// genéticas (AST Hash y Dependency Hash) con los valores esperados, mitigando
/// fallos TOCTOU (*Time-of-Check to Time-of-Use*).
pub fn examinar_estado_matriz(name: &str, expected_ast: u64, expected_dep: u64) -> MatrixStatus {
    let out_dir = match std::env::var("OUT_DIR") {
        Ok(d) => d,
        Err(_) => return MatrixStatus::Missing,
    };
    let ruta = Path::new(&out_dir)
        .join("foundry")
        .join(format!("{}.matrix", name));

    let mut file = match File::open(&ruta) {
        Ok(f) => f,
        Err(_) => return MatrixStatus::Missing,
    };

    let mut header = [0u8; HEADER_SIZE];
    if file.read_exact(&mut header).is_err() {
        return MatrixStatus::Corrupted;
    }
    if &header[0..4] != b"MATR" {
        return MatrixStatus::Corrupted;
    }

    let target_endian =
        std::env::var("CARGO_CFG_TARGET_ENDIAN").unwrap_or_else(|_| "little".into());

    let read_u64 = |start: usize| -> u64 {
        let mut b = [0u8; 8];
        b.copy_from_slice(&header[start..start + 8]);
        if target_endian == "little" {
            u64::from_le_bytes(b)
        } else {
            u64::from_be_bytes(b)
        }
    };

    let found_ast = read_u64(7);
    let found_dep = read_u64(15);

    if found_ast != expected_ast {
        return MatrixStatus::Obsolete("Código fuente modificado.".into());
    }
    if found_dep != expected_dep {
        return MatrixStatus::Obsolete("Dependencias o Arquitectura alteradas.".into());
    }

    MatrixStatus::Valid
}

/// Esculpe el negativo molecular `.matrix` inyectando la firma genética en su cabecera física.
///
/// Serializa el objeto producido por `f` con Bincode, prependedo la cabecera de 31 bytes
/// con magia, endianness, versión, hashes y longitud del payload. Utiliza escritura
/// atómica mediante archivo temporal + `rename` para garantizar integridad.
///
/// # Layout de cabecera (31 bytes)
///
/// | Offset | Tamaño | Campo            |
/// |--------|--------|------------------|
/// | 0..4   | 4 B    | MAGIC `"MATR"`   |
/// | 4      | 1 B    | Endianness mark  |
/// | 5..7   | 2 B    | Version (u16)    |
/// | 7..15  | 8 B    | AST Hash (u64)   |
/// | 15..23 | 8 B    | Dep Hash (u64)   |
/// | 23..31 | 8 B    | Payload Len (u64)|
pub fn execute_matrix_capture<T>(name: &str, ast_hash: u64, dep_hash: u64, f: fn() -> T)
where
    T: serde::Serialize,
{
    let objeto = f();
    let target = std::env::var("TARGET").unwrap_or_default();

    // Cross-Compilation: Determinar orientación de bytes del procesador objetivo
    let es_big_endian = ["mips", "powerpc", "sparc", "s390x"]
        .iter()
        .any(|arch| target.starts_with(arch));

    let payload = if es_big_endian {
        bincode::options()
            .with_big_endian()
            .serialize(&objeto)
            .unwrap()
    } else {
        bincode::options()
            .with_little_endian()
            .serialize(&objeto)
            .unwrap()
    };

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let carpeta = Path::new(&out_dir).join("foundry");
    fs::create_dir_all(&carpeta).unwrap();
    let ruta_final = carpeta.join(format!("{}.matrix", name));
    let ruta_temporal = ruta_final.with_extension("tmp");

    {
        let mut tmp_file = File::create(&ruta_temporal).unwrap();

        // MAGIC (4 bytes)
        tmp_file.write_all(b"MATR").unwrap();

        // Endianness mark (1 byte)
        tmp_file
            .write_all(&[if es_big_endian { 0x02 } else { 0x01 }])
            .unwrap();

        // Version core (2 bytes, u16 nativo del target)
        let version_bytes: [u8; 2] = if es_big_endian {
            1u16.to_be_bytes()
        } else {
            1u16.to_le_bytes()
        };
        tmp_file.write_all(&version_bytes).unwrap();

        // Helper para escribir u64 en endianness del target
        let write_u64 = |w: &mut File, v: u64| {
            let b = if es_big_endian {
                v.to_be_bytes()
            } else {
                v.to_le_bytes()
            };
            w.write_all(&b).unwrap();
        };

        // AST Hash (8 bytes)
        write_u64(&mut tmp_file, ast_hash);

        // Dependency Hash (8 bytes)
        write_u64(&mut tmp_file, dep_hash);

        // Payload Len (8 bytes)
        write_u64(&mut tmp_file, payload.len() as u64);

        // Payload (longitud variable)
        tmp_file.write_all(&payload).unwrap();

        // Asegurar persistencia física en el medio de almacenamiento
        tmp_file.sync_all().unwrap();
    }

    // Intercambio atómico del archivo en el sistema de archivos
    fs::rename(&ruta_temporal, &ruta_final).unwrap();
}
