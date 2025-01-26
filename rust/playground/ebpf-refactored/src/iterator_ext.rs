// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

pub struct ConstTake<I: Iterator, const N: usize> {
    iter: I,
    index: usize,
}

impl<I: Iterator, const N: usize> ConstTake<I, N> {
    pub fn new(iter: I) -> Self {
        Self { iter, index: 0 }
    }
}

impl<I: Iterator, const N: usize> Iterator for ConstTake<I, N> {
    type Item = I::Item;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index == N {
            None
        } else {
            self.index += 1;
            self.iter.next()
        }
    }
}

pub trait IteratorExt: Sized + Iterator {
    fn const_take<const N: usize>(self) -> ConstTake<Self, N>;
}

impl<I: Iterator> IteratorExt for I {
    fn const_take<const N: usize>(self) -> ConstTake<Self, N> {
        ConstTake::new(self)
    }
}
