/// Estructura de datos interna que representa el encabezado físico.
pub struct PatternMetadata {
    pub ast_hash: u64,
    pub type_hash: u64,
    pub dependency_hash: u64,
}