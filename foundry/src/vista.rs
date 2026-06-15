use rkyv::validation::validators::DefaultValidator;
use std::marker::PhantomData;

pub enum Pipeline<T: rkyv::Archive> {
    /// Modo Producción: Apunta a los bytes estáticos inyectados en .rodata.
    Forged(&'static [u8], PhantomData<T>),
    /// Modo Desarrollo: Contiene los bytes ya cocinados en RAM en el paso inicial.
    Live(Vec<u8>, PhantomData<T>),
}

impl<T: rkyv::Archive + 'static> Pipeline<T>
where
    rkyv::Archived<T>: for<'b> rkyv::CheckBytes<DefaultValidator<'b>> + 'static,
{
    /// .map() inmediato, limpio y con inferencia perfecta.
    #[inline(always)]
    pub fn map<U, F>(self, f: F) -> U
    where
        F: FnOnce(&rkyv::Archived<T>) -> U,
    {
        let archived_t = match &self {
            Pipeline::Forged(bytes, _) => {
                let payload = &bytes[40..];
                unsafe { rkyv::archived_root::<T>(payload) }
            }
            Pipeline::Live(bytes, _) => unsafe { rkyv::archived_root::<T>(bytes) },
        };

        f(archived_t)
    }
}
