/// Contrato estático para patrones de inyección atómica.
///
/// Implementado automáticamente por #[pattern]. No implementar a mano.
pub trait Pattern {
    type Output;
    const AST_HASH: u64;
    const DEPENDENCY_HASH: u64;
    const BAKED_TEMPLATE: Option<&'static [u8]>;
    fn execute() -> Self::Output;
}