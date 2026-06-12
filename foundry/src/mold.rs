use crate::mold_runtime::cast_from_matrix;

/// Trait intermedio para proyectar de forma estable el tipo de retorno en Rust estable.
pub trait MoldPattern {
    type Output;
    fn ejecutar(&self) -> Self::Output;
}

// Implementación automática para cualquier puntero de función puro
impl<T> MoldPattern for fn() -> T {
    type Output = T;

    #[inline(always)]
    fn ejecutar(&self) -> T {
        (self)()
    }
}

/// Envoltura universal purista de cero coste en runtime.
///
/// El tipo `F` identifica unívocamente la función pura en el sistema de tipos.
pub struct Mold<F> {
    funcion_fallback: F,
    bytes_inyectados: Option<&'static [u8]>,
}

impl<F> Mold<F>
where
    F: MoldPattern + Copy,
    <F as MoldPattern>::Output: serde::de::DeserializeOwned,
{
    /// Constructor interno de cero asignación en el Heap.
    #[inline(always)]
    #[doc(hidden)]
    pub const fn new_internal(
        funcion_fallback: F,
        bytes_inyectados: Option<&'static [u8]>,
    ) -> Self {
        Self {
            funcion_fallback,
            bytes_inyectados,
        }
    }

    /// Deserializa desde los bytes estáticos inyectados o ejecuta la función original.
    #[inline(always)]
    pub fn cast(&self) -> <F as MoldPattern>::Output {
        if let Some(matrix_bytes) = self.bytes_inyectados {
            match cast_from_matrix::<<F as MoldPattern>::Output>(matrix_bytes) {
                Ok(objeto) => return objeto,
                Err(e) => panic!("foundry: cast_from_matrix falló: {:?}", e),
            }
        }

        // Ejecutamos el fallback a través de nuestro trait estable
        self.funcion_fallback.ejecutar()
    }

    /// Comprobación booleana en tiempo de compilación.
    #[inline(always)]
    pub const fn is_baked(&self) -> bool {
        self.bytes_inyectados.is_some()
    }
}
