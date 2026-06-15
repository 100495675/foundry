#[doc(hidden)]
pub struct DefaultRouter;

impl DefaultRouter {
    #[inline(always)]
    pub fn __foundry_get_matrix(&self) -> Option<&'static [u8]> {
        None
    }
}
