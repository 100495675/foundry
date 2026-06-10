//! # foundry-internal
//!
//! Tipos privados de control interno para el ecosistema `foundry`.

/// Trait de contrato estático para patrones de inyección atómica.
pub trait Pattern {
    /// Tipo de retorno del patrón (el tipo materializado por `Mold::cast()`).
    type Output;

    /// Firma sintáctica del código fuente del cuerpo de la función.
    const AST_HASH: u64;

    /// Firma del entorno de dependencias y arquitectura.
    const DEPENDENCY_HASH: u64;

    /// Bytes del artefacto pre-horneado, disponibles solo en compilación de producción.
    const BAKED_TEMPLATE: Option<&'static [u8]>;

    /// Ejecuta la lógica original del patrón como mecanismo de fallback.
    fn execute() -> Self::Output;
}
