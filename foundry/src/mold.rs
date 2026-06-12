use crate::mold_runtime::cast_from_matrix;

pub struct Mold<T> {
    funcion_fallback: Box<dyn Fn() -> T>,
    bytes_inyectados: Option<&'static [u8]>,
}

impl<T> Mold<T>
where
    T: serde::de::DeserializeOwned,
{
    #[inline(always)]
    #[doc(hidden)]
    pub fn new_internal(
        funcion_fallback: Box<dyn Fn() -> T>,
        bytes_inyectados: Option<&'static [u8]>,
    ) -> Self {
        Self {
            funcion_fallback,
            bytes_inyectados,
        }
    }

    #[inline(always)]
    pub fn cast(&self) -> T {
        if let Some(matrix_bytes) = self.bytes_inyectados {
            match cast_from_matrix::<T>(matrix_bytes) {
                Ok(objeto) => return objeto,
                Err(e) => panic!("foundry: cast_from_matrix falló: {:?}", e),
            }
        }
        (self.funcion_fallback)()
    }

    #[inline(always)]
    pub const fn is_baked(&self) -> bool {
        self.bytes_inyectados.is_some()
    }
}
