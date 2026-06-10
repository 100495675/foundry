use crate::internal::Pattern;
use crate::mold_runtime::cast_from_matrix;
use crate::shape::Shape;

/// Contenedor `Mold<F>` — Envoltorio de inyección atómica para un patrón `F`.
pub struct Mold<F> {
    _pattern_token: std::marker::PhantomData<F>,
}

impl<F, T> Mold<F>
where
    F: Pattern<Output = T>,
    T: Shape + serde::de::DeserializeOwned,
{
    /// Construye un nuevo molde a partir del patrón (ZST) proporcionado.
    ///
    /// Al exigir `Copy`, evitamos que las funciones constantes evalúen destructores (E0493).
    #[inline(always)]
    pub const fn new(_pattern: F) -> Self
    where
        F: Copy,
    {
        Self {
            _pattern_token: std::marker::PhantomData,
        }
    }

    /// Despacha la materialización del objeto. Optimizado mediante poda de ramas muertas.
    #[inline(always)]
    pub fn cast(&self) -> T {
        if let Some(matrix_bytes) = F::BAKED_TEMPLATE {
            unsafe {
                if let Ok(objeto) = cast_from_matrix::<T>(matrix_bytes) {
                    return objeto;
                }
            }
        }
        F::execute()
    }

    #[inline(always)]
    pub const fn is_baked(&self) -> bool {
        F::BAKED_TEMPLATE.is_some()
    }
}

impl<F> Clone for Mold<F> {
    fn clone(&self) -> Self {
        Self {
            _pattern_token: std::marker::PhantomData,
        }
    }
}

impl<F> Copy for Mold<F> {}

impl<F: Pattern> std::fmt::Debug for Mold<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Mold")
            .field("baked", &F::BAKED_TEMPLATE.is_some())
            .finish()
    }
}
