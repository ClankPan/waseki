// use num_traits::{One, Zero};
// use std::iter::{Product, Sum};
//
// use crate::L;

// // by-value: Iterator<Item = L>
// impl<'id, T> Sum for L<'id, T>
// where
//     T: Copy + Default + PartialEq + One + Zero,
// {
//     fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
//         iter.fold(Self::new(), |acc, x| acc + x)
//     }
// }
//
// // by-ref: Iterator<Item = &L>
// impl<'id, 'a, T> Sum<&'a L<'id, T>> for L<'id, T>
// where
//     T: Copy + Default + PartialEq + One + Zero,
// {
//     fn sum<I: Iterator<Item = &'a L<'id, T>>>(iter: I) -> Self {
//         iter.fold(Self::zero(), |acc, x| acc + x)
//     }
// }
//
// // Product も同様
// impl<'id, T> Product for L<'id, T>
// where
//     T: Copy + Default + PartialEq + One + Zero,
// {
//     fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
//         iter.fold(Self::one(), |acc, x| acc * x)
//     }
// }
//
// impl<'id, 'a, T> Product<&'a L<'id, T>> for L<'id, T>
// where
//     T: Copy + Default + PartialEq + One + Zero,
// {
//     fn product<I: Iterator<Item = &'a L<'id, T>>>(iter: I) -> Self {
//         iter.fold(Self::one(), |acc, x| acc * x)
//     }
// }
