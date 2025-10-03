use std::ops::{Add, Mul};

use num_traits::{One, Zero};

use crate::{C, L, Q};

/* ========= 演算子トレイト ========= */
// &L + &L -> L
impl<'id, T> Add for &L<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    type Output = L<'id, T>;
    fn add(self, rhs: Self) -> Self::Output {
        *self + *rhs
    }
}

// &L * &L -> Q
impl<'id, T> Mul for &L<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    type Output = Q<'id, T>;
    fn mul(self, rhs: Self) -> Self::Output {
        *self * *rhs
    }
}

// &L + &Q -> Q
impl<'id, T> Add<&Q<'id, T>> for &L<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    type Output = Q<'id, T>;
    #[inline]
    fn add(self, rhs: &Q<'id, T>) -> Self::Output {
        *rhs + *self
    }
}

// &L * &Q -> Q
impl<'id, T> Mul<&Q<'id, T>> for &L<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    type Output = Q<'id, T>;
    #[inline]
    fn mul(self, rhs: &Q<'id, T>) -> Self::Output {
        *rhs * *self
    }
}

// &Q + &L -> Q
impl<'id, T> Add<&L<'id, T>> for &Q<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    type Output = Q<'id, T>;
    fn add(self, rhs: &L<'id, T>) -> Self::Output {
        *self + *rhs
    }
}

// &Q * &L -> Q
impl<'id, T> Mul<&L<'id, T>> for &Q<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    type Output = Q<'id, T>;
    #[inline]
    fn mul(self, rhs: &L<'id, T>) -> Self::Output {
        *self * *rhs
    }
}

// &Q * &Q -> Q
impl<'id, T> Mul for &Q<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    type Output = Q<'id, T>;
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        *self * *rhs
    }
}

// &Q + &Q -> L
impl<'id, T> Add for &Q<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    type Output = L<'id, T>;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        *self + *rhs
    }
}

// &u128 * &L -> L
impl<'id, T> Mul<&L<'id, T>> for &u128
where
    T: Copy + One + Zero + PartialEq + From<u128> + Default,
{
    type Output = L<'id, T>;
    #[inline]
    fn mul(self, rhs: &L<'id, T>) -> Self::Output {
        *self * *rhs
    }
}

// &L * &u128 -> L
impl<'id, T> Mul<&u128> for &L<'id, T>
where
    T: Copy + One + Zero + PartialEq + From<u128> + Default,
{
    type Output = L<'id, T>;
    #[inline]
    fn mul(self, rhs: &u128) -> Self::Output {
        *self * *rhs
    }
}

// &u128 + &L -> L
impl<'id, T> Add<&L<'id, T>> for &u128
where
    T: Copy + One + Zero + PartialEq + From<u128> + Default,
{
    type Output = L<'id, T>;
    #[inline]
    fn add(self, rhs: &L<'id, T>) -> Self::Output {
        *self + *rhs
    }
}

// &L + &u128 -> L
impl<'id, T> Add<&u128> for &L<'id, T>
where
    T: Copy + One + Zero + PartialEq + From<u128> + Default,
{
    type Output = L<'id, T>;
    #[inline]
    fn add(self, rhs: &u128) -> Self::Output {
        *self + *rhs
    }
}

// &u128 * &Q -> Q
impl<'id, T> Mul<&Q<'id, T>> for &u128
where
    T: Copy + One + Zero + PartialEq + From<u128> + Default,
{
    type Output = Q<'id, T>;
    #[inline]
    fn mul(self, rhs: &Q<'id, T>) -> Self::Output {
        *self * *rhs
    }
}

// &Q * &u128 -> Q
impl<'id, T> Mul<&u128> for &Q<'id, T>
where
    T: Copy + One + Zero + PartialEq + From<u128> + Default,
{
    type Output = Q<'id, T>;
    #[inline]
    fn mul(self, rhs: &u128) -> Self::Output {
        *self * *rhs
    }
}

// &u128 + &Q -> Q
impl<'id, T> Add<&Q<'id, T>> for &u128
where
    T: Copy + One + Zero + PartialEq + From<u128> + Default,
{
    type Output = Q<'id, T>;
    #[inline]
    fn add(self, rhs: &Q<'id, T>) -> Self::Output {
        *self + *rhs
    }
}

// &Q + &u128 -> Q
impl<'id, T> Add<&u128> for &Q<'id, T>
where
    T: Copy + One + Zero + PartialEq + From<u128> + Default,
{
    type Output = Q<'id, T>;
    #[inline]
    fn add(self, rhs: &u128) -> Self::Output {
        *self + *rhs
    }
}

// --- C<T>（孤児回避ラッパ）の参照版 --- //

// &C * &L -> L
impl<'id, T> Mul<&L<'id, T>> for &C<T>
where
    T: Copy + One + Zero + PartialEq + From<u128> + Default,
{
    type Output = L<'id, T>;
    #[inline]
    fn mul(self, rhs: &L<'id, T>) -> Self::Output {
        *self * *rhs
    }
}

// &L * &C -> L
impl<'id, T> Mul<&C<T>> for &L<'id, T>
where
    T: Copy + One + Zero + PartialEq + From<u128> + Default,
{
    type Output = L<'id, T>;
    #[inline]
    fn mul(self, rhs: &C<T>) -> Self::Output {
        *self * *rhs
    }
}

// &C + &L -> L
impl<'id, T> Add<&L<'id, T>> for &C<T>
where
    T: Copy + One + Zero + PartialEq + From<u128> + Default,
{
    type Output = L<'id, T>;
    #[inline]
    fn add(self, rhs: &L<'id, T>) -> Self::Output {
        *self + *rhs
    }
}

// &L + &C -> L
impl<'id, T> Add<&C<T>> for &L<'id, T>
where
    T: Copy + One + Zero + PartialEq + From<u128> + Default,
{
    type Output = L<'id, T>;
    #[inline]
    fn add(self, rhs: &C<T>) -> Self::Output {
        *self + *rhs
    }
}

// &C * &Q -> Q
impl<'id, T> Mul<&Q<'id, T>> for &C<T>
where
    T: Copy + One + Zero + PartialEq + From<u128> + Default,
{
    type Output = Q<'id, T>;
    #[inline]
    fn mul(self, rhs: &Q<'id, T>) -> Self::Output {
        *self * *rhs
    }
}

// &Q * &C -> Q
impl<'id, T> Mul<&C<T>> for &Q<'id, T>
where
    T: Copy + One + Zero + PartialEq + From<u128> + Default,
{
    type Output = Q<'id, T>;
    #[inline]
    fn mul(self, rhs: &C<T>) -> Self::Output {
        *self * *rhs
    }
}

// &C + &Q -> Q
impl<'id, T> Add<&Q<'id, T>> for &C<T>
where
    T: Copy + One + Zero + PartialEq + From<u128> + Default,
{
    type Output = Q<'id, T>;
    #[inline]
    fn add(self, rhs: &Q<'id, T>) -> Self::Output {
        *self + *rhs
    }
}

// &Q + &C -> Q
impl<'id, T> Add<&C<T>> for &Q<'id, T>
where
    T: Copy + One + Zero + PartialEq + From<u128> + Default,
{
    type Output = Q<'id, T>;
    #[inline]
    fn add(self, rhs: &C<T>) -> Self::Output {
        *self + *rhs
    }
}
