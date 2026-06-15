#![no_std]

/// 🟠 ALTO 3: Offset de cabecera estructural para absorber metadatos y padding de hardware.
/// Este número viene del tamaño del prefijo introducido por el framework original en los tests
/// de captura ("MATR" + versión + hashes + longitud), redondeado para asegurar alineación de 64 bits.
pub const MATRIX_HEADER_SIZE: usize = 40;

/// Metadatos de control para futuras validaciones de evolución del AST (Deuda técnica mitigada)
pub struct PatternMetadata {
    pub ast_hash: u64,
    pub type_hash: u64,
}