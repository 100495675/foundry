use crate::mold_runtime::cast_from_matrix;
use foundry_core::Pattern;

pub struct Mold<F> {
    _pattern_token: std::marker::PhantomData<F>,
}

impl<F, T> Mold<F>
where
    F: Pattern<Output = T>,
    T: serde::de::DeserializeOwned,
{
    #[inline(always)]
    pub const fn new(_pattern: F) -> Self
    where
        F: Copy,
    {
        Self {
            _pattern_token: std::marker::PhantomData,
        }
    }

    #[inline(always)]
    pub fn cast(&self) -> T {
        if let Some(matrix_bytes) = F::BAKED_TEMPLATE {
            match cast_from_matrix::<T>(matrix_bytes) {
                Ok(objeto) => return objeto,
                Err(e) => panic!("foundry: cast_from_matrix falló: {:?}", e),
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
