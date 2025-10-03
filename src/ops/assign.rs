use num_traits::{One, Zero};
use std::ops::AddAssign;

use crate::{L, Q};

use super::binary::C;

// L += L
impl<'id, T> AddAssign for L<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    fn add_assign(&mut self, rhs: Self) {
        *self = &*self + &rhs;
    }
}

// L += &L
impl<'id, T> AddAssign<&L<'id, T>> for L<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    #[inline]
    fn add_assign(&mut self, rhs: &Self) {
        *self = &*self + rhs;
    }
}

// Q += L
impl<'id, T> AddAssign<L<'id, T>> for Q<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    fn add_assign(&mut self, rhs: L<'id, T>) {
        *self = &*self + &rhs;
    }
}

// Q += &L
impl<'id, T> AddAssign<&L<'id, T>> for Q<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    fn add_assign(&mut self, rhs: &L<'id, T>) {
        *self = &*self + rhs;
    }
}

// Q += Q
impl<'id, T> AddAssign for Q<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    fn add_assign(&mut self, rhs: Self) {
        *self = &*self + &rhs;
    }
}

// Q += &Q
impl<'id, T> AddAssign<&Q<'id, T>> for Q<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    fn add_assign(&mut self, rhs: &Self) {
        *self = &*self + rhs;
    }
}

// L += T
impl<'id, T> AddAssign<T> for L<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    fn add_assign(&mut self, rhs: T) {
        *self = &*self + &C(rhs);
    }
}

// L += &T
impl<'id, T> AddAssign<&T> for L<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    #[inline]
    fn add_assign(&mut self, rhs: &T) {
        *self = &*self + &C(*rhs);
    }
}
