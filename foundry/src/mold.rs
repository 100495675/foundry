pub trait Pour {
    type Output: rkyv::Archive;
    fn pour(&self) -> Self::Output;
}

impl<R, G> Pour for G
where
    G: Fn() -> R + Copy,
    R: rkyv::Archive,
{
    type Output = R;

    #[inline(always)]
    fn pour(&self) -> R {
        (self)()
    }
}

pub struct Mold<F, Target> {
    pub fallback_function: F,
    pub injected_bytes: Option<&'static [u8]>,
    pub _marker: std::marker::PhantomData<Target>,
}

impl<F, T> Mold<F, T> {
    #[inline(always)]
    #[doc(hidden)]
    pub const fn new_internal(fallback_function: F, injected_bytes: Option<&'static [u8]>) -> Self {
        Self {
            fallback_function,
            injected_bytes,
            _marker: std::marker::PhantomData,
        }
    }
}
