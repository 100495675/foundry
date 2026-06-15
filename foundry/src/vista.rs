use foundry_core::PatternMetadata;

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
pub fn validar_matriz_auditoria(bytes: &[u8], expected_meta: &PatternMetadata) {
    if bytes.len() < 40 {
        panic!("foundry: Violación física. Tamaño de buffer insuficiente para cabecera densa.");
    }

    let header_bytes: &[u8; 40] = unsafe { &*(bytes.as_ptr() as *const [u8; 40]) };
    let bin_header = unsafe { &*(header_bytes.as_ptr() as *const PatternMetadata) };

    if bin_header.magic != expected_meta.magic {
        panic!("foundry: Fallo de firma mágica de hardware corrupta.");
    }
    if bin_header.name_hash != expected_meta.name_hash {
        panic!("foundry: Violación de identidad de función. El name_hash no coincide.");
    }
    if bin_header.type_hash != expected_meta.type_hash {
        panic!("foundry: Violación estructural. El layout del tipo de retorno ha mutado.");
    }
    if bin_header.version != expected_meta.version {
        panic!("foundry: Conflicto de versión del compilador de matrices.");
    }
}
