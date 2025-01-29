// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

pub trait EbpfBoundsCheck {
    /// # SAFETY
    ///
    /// Bound must be a power of two
    unsafe fn bounded<const N: usize>(self) -> Option<Self>
    where
        Self: Sized;
}

#[cfg(feature = "bounds-check")]
impl EbpfBoundsCheck for usize {
    #[inline(always)]
    unsafe fn bounded<const N: usize>(self) -> Option<Self> {
        let this = self & ((N << 1) - 1);
        if this & N != 0 {
            None
        } else {
            Some(this & (N - 1))
        }
    }
}

#[cfg(not(feature = "bounds-check"))]
impl EbpfBoundsCheck for usize {
    #[inline(always)]
    unsafe fn bounded<const N: usize>(self) -> Option<Self> {
        Some(self & (N - 1))
    }
}
