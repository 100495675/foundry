// foundry-core/src/lib.rs
#![no_std]

pub const MATRIX_MAGIC: &[u8; 4] = b"MATR";
pub const MATRIX_VERSION: u8 = 0x02;
pub const MATRIX_HEADER_SIZE: usize = 40;

#[repr(C)] // 👈 Saneado: Alineamiento natural seguro para referencias nativas
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub struct PatternMetadata {
    pub name_hash: u64,   // 0..8   (8 bytes)
    pub type_hash: u64,   // 8..16  (8 bytes)
    pub payload_len: u64, // 16..24 (8 bytes)
    pub reserved_1: u64,  // 24..32 (8 bytes)
    pub magic: [u8; 4],   // 32..36 (4 bytes)
    pub schema_ver: u16,  // 36..38 (2 bytes)
    pub version: u8,      // 38..39 (1 byte)
    pub padding: u8,      // 39..40 (1 byte)
} // Total: Exactamente 40 bytes.
