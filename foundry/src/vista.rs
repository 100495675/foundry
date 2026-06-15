// foundry/src/vista.rs
use foundry_core::{PatternMetadata, MATRIX_MAGIC, MATRIX_VERSION};

#[derive(Clone)]
pub enum Pipeline<R: ::rkyv::Archive> {
    Forged(&'static [u8], PatternMetadata, std::marker::PhantomData<R>),
    Live(Vec<u8>, std::marker::PhantomData<R>),
}

impl<R: ::rkyv::Archive> Pipeline<R>
where
    R::Archived: 'static,
{
    #[inline(always)]
    pub fn map<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&R::Archived) -> T,
    {
        match self {
            Pipeline::Forged(bytes, _, _) => {
                let payload_bytes = &bytes[40..];
                let archived = unsafe { ::rkyv::archived_root::<R>(payload_bytes) };
                f(archived)
            }
            Pipeline::Live(bytes, _) => {
                let payload_bytes = &bytes[..];
                let archived = unsafe { ::rkyv::archived_root::<R>(payload_bytes) };
                f(archived)
            }
        }
    }
}

#[inline(always)]
pub fn validar_matriz_auditoria(bytes: &[u8], _expected_meta: &PatternMetadata) -> bool {
    if bytes.len() < 40 {
        return false;
    }

    let bin_header = unsafe { std::ptr::read_unaligned(bytes.as_ptr() as *const PatternMetadata) };

    if bin_header.magic != *MATRIX_MAGIC || bin_header.version != MATRIX_VERSION {
        return false;
    }

    true
}
