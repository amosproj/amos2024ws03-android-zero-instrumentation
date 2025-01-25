pub trait EbpfBoundsCheck {
    /// # SAFETY
    ///
    /// Bound must be a power of two
    unsafe fn bounded(self, bound: usize) -> Option<Self>
    where
        Self: Sized;
}

#[cfg(feature = "bounds-check")]
impl EbpfBoundsCheck for usize {
    #[inline(always)]
    unsafe fn bounded(self, bound: usize) -> Option<Self> {
        let this = self & ((bound << 1) - 1);
        if this & bound != 0 {
            return None;
        } else {
            return Some(this & (bound - 1));
        }
    }
}

#[cfg(not(feature = "bounds-check"))]
impl EbpfBoundsCheck for usize {
    #[inline(always)]
    unsafe fn bounded(self, bound: usize) -> Option<Self> {
        Some(self & (bound - 1))
    }
}
