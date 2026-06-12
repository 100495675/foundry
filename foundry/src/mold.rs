use crate::mold_runtime::cast_from_matrix;

/// Trait interno para proyectar y resolver de forma estática el tipo de retorno
/// de cualquier Function Item o puntero plano en Rust Estable.
pub trait MoldPattern {
    type Output;
    fn ejecutar(&self) -> Self::Output;
}

// Implementación automática universal para punteros de función puros
impl<T> MoldPattern for fn() -> T {
    type Output = T;

    #[inline(always)]
    fn ejecutar(&self) -> T {
        (self)()
    }
}

/// Envoltura universal purista de cero coste en runtime.
///
/// El tipo `F` mapea con precisión matemática la identidad única de la función.
pub struct Mold<F> {
    funcion_fallback: F,
    bytes_inyectados: Option<&'static [u8]>,
}

// Implementación de Clone explícita para evitar restricciones virales
impl<F: Clone> Clone for Mold<F> {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self {
            funcion_fallback: self.funcion_fallback.clone(),
            bytes_inyectados: self.bytes_inyectados,
        }
    }
}

// Implementación de Copy para permitir el paso por valor nativo sin coste
impl<F: Copy> Copy for Mold<F> {}

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

        // Ejecución directa del fallback, 100% inlinable por el compilador
        self.funcion_fallback.ejecutar()
    }

    /// Comprobación booleana en tiempo de compilación.
    #[inline(always)]
    pub const fn is_baked(&self) -> bool {
        self.bytes_inyectados.is_some()
    }
}
