use foundry_core::MATRIX_HEADER_SIZE;
use rkyv::validation::validators::DefaultValidator;
use std::marker::PhantomData;

pub enum Pipeline<T: rkyv::Archive> {
    Forged(&'static [u8], PhantomData<T>),
    Live(Vec<u8>, PhantomData<T>),
}

impl<T: rkyv::Archive + 'static> Pipeline<T>
where
    rkyv::Archived<T>: for<'b> rkyv::CheckBytes<DefaultValidator<'b>> + 'static,
{
    #[inline(always)]
    pub fn map<U, F>(self, f: F) -> U
    where
        F: FnOnce(&rkyv::Archived<T>) -> U,
    {
        let archived_t = match &self {
            Pipeline::Forged(bytes, _) => {
                // Ahora el offset salta exactamente 40 bytes, respetando el padding de hardware
                let payload = &bytes[MATRIX_HEADER_SIZE..];

                #[cfg(debug_assertions)]
                {
                    rkyv::check_archived_root::<T>(payload).expect(
                        "foundry: Corrupción estructural detectada en la sección .rodata forjada",
                    )
                }
                #[cfg(not(debug_assertions))]
                {
                    unsafe { rkyv::archived_root::<T>(payload) }
                }
            }
            Pipeline::Live(bytes, _) => {
                // El carril live no lleva la cabecera del archivo en disco, opera directo
                #[cfg(debug_assertions)]
                {
                    rkyv::check_archived_root::<T>(bytes)
                        .expect("foundry: Corrupción estructural detectada en la rampa de memoria dinámica (Live)")
                }
                #[cfg(not(debug_assertions))]
                {
                    unsafe { rkyv::archived_root::<T>(bytes) }
                }
            }
        };

        f(archived_t)
    }
}
