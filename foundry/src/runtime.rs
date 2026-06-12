use bincode::Options;

/// Standardized configuration for ultra-fast bincode deserialization.
#[inline(always)]
#[doc(hidden)]
pub fn bincode_options() -> impl bincode::Options {
    bincode::options()
        .with_little_endian()
        .with_fixint_encoding()
        .allow_trailing_bytes()
}

/// Extracts the binary payload by skipping the rigid foundry matrix header.
#[inline(always)]
#[doc(hidden)]
pub fn cast_from_matrix<T: serde::de::DeserializeOwned>(
    bytes: &'static [u8],
) -> Result<T, bincode::Error> {
    // Skip the 39 bytes of the rigid forge header (Magic, Hashes, Sizes)
    let payload = &bytes[39..];
    bincode_options().deserialize(payload)
}

/// Global fallback router trait for Autoref method resolution.
#[doc(hidden)]
pub trait FoundryFallbackRouter {
    #[inline(always)]
    fn __foundry_get_matrix(&self, _target_ptr: usize) -> Option<&'static [u8]> {
        None
    }
}

// Lowest priority fallback: applies to a reference of a reference of a flat function pointer.
impl<T> FoundryFallbackRouter for &&fn() -> T {}
