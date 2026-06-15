#![no_std]

pub const MATRIX_MAGIC: &[u8; 4] = b"MATR";
pub const MATRIX_VERSION: u8 = 0x02;
pub const MATRIX_HEADER_SIZE: usize = 40;

/// Estructura física alineada por hardware de 40 bytes.
#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub struct PatternMetadata {
    pub name_hash: u64,   // 0..8   - Identifica el nombre inequívoco de la función
    pub type_hash: u64,   // 8..16  - Identifica la firma estructural de rkyv
    pub payload_len: u64, // 16..24 - Tamaño del segmento serializado en bytes
    pub reserved: u64,    // 24..32 - Reservado para dependency_hash estructural en v3
    pub magic: [u8; 4],   // 32..36 - b"MATR"
    pub schema_ver: u16,  // 36..38 - Versión del esquema de metadatos (esperado: 1)
    pub version: u8,      // 38..39 - Versión de la suite Foundry
    pub padding: u8,      // 39..40
}
