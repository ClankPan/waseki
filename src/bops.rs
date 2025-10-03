use std::ops::{Add, Mul};

use num_traits::{One, Zero};

use crate::{
    L, Q, l_add_l, l_mul_l, q_add_l, q_add_q, q_mul_l, q_mul_q, t_add_l, t_add_q, t_mul_l, t_mul_q,
};

/* ========= 演算子トレイト ========= */
// L + L -> L
impl<'id, T> Add for L<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    type Output = L<'id, T>;
    fn add(self, rhs: Self) -> Self::Output {
        l_add_l(self, rhs)
    }
}

// L * L -> Q
impl<'id, T> Mul for L<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    type Output = Q<'id, T>;
    fn mul(self, rhs: Self) -> Self::Output {
        l_mul_l(self, rhs)
    }
}

// L + Q -> Q
impl<'id, T> Add<Q<'id, T>> for L<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    type Output = Q<'id, T>;
    #[inline]
    fn add(self, rhs: Q<'id, T>) -> Self::Output {
        q_add_l(rhs, self)
    }
}

// L * Q -> Q
impl<'id, T> Mul<Q<'id, T>> for L<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    type Output = Q<'id, T>;
    #[inline]
    fn mul(self, rhs: Q<'id, T>) -> Self::Output {
        // Q * L の実装をそのまま利用
        q_mul_l(rhs, self)
    }
}

// Q + L -> Q
impl<'id, T> Add<L<'id, T>> for Q<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    type Output = Q<'id, T>;
    fn add(self, rhs: L<'id, T>) -> Self::Output {
        q_add_l(self, rhs)
    }
}

// Q * L -> Q
impl<'id, T> Mul<L<'id, T>> for Q<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    type Output = Q<'id, T>;
    fn mul(self, rhs: L<'id, T>) -> Self::Output {
        q_mul_l(self, rhs)
    }
}

// Q * Q -> Q
impl<'id, T> Mul for Q<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    type Output = Q<'id, T>;
    fn mul(self, rhs: Self) -> Self::Output {
        q_mul_q(self, rhs)
    }
}

// Q + Q -> Q
impl<'id, T: Clone> Add for Q<'id, T>
where
    T: Copy + Default + PartialEq + One + Zero,
{
    type Output = L<'id, T>;
    fn add(self, rhs: Self) -> Self::Output {
        q_add_q(self, rhs)
    }
}

// u128 * L -> L
impl<'id, T> Mul<L<'id, T>> for u128
where
    T: Copy + One + Zero + PartialEq + From<u128> + Default,
{
    type Output = L<'id, T>;
    #[inline]
    fn mul(self, rhs: L<'id, T>) -> Self::Output {
        t_mul_l(self.into(), rhs)
    }
}

// L * u128 -> L
impl<'id, T> Mul<u128> for L<'id, T>
where
    T: Copy + One + Zero + PartialEq + From<u128> + Default,
{
    type Output = L<'id, T>;
    #[inline]
    fn mul(self, rhs: u128) -> Self::Output {
        t_mul_l::<_>(rhs.into(), self)
    }
}

// u128 + L -> L
impl<'id, T> Add<L<'id, T>> for u128
where
    T: Copy + One + Zero + PartialEq + From<u128> + Default,
{
    type Output = L<'id, T>;
    #[inline]
    fn add(self, rhs: L<'id, T>) -> Self::Output {
        t_add_l(self.into(), rhs)
    }
}

// L + u128 -> L
impl<'id, T> Add<u128> for L<'id, T>
where
    T: Copy + One + Zero + PartialEq + From<u128> + Default,
{
    type Output = L<'id, T>;
    #[inline]
    fn add(self, rhs: u128) -> Self::Output {
        t_add_l(rhs.into(), self)
    }
}

// u128 * Q -> Q
impl<'id, T> Mul<Q<'id, T>> for u128
where
    T: Copy + One + Zero + PartialEq + From<u128> + Default,
{
    type Output = Q<'id, T>;
    #[inline]
    fn mul(self, rhs: Q<'id, T>) -> Self::Output {
        t_mul_q(self.into(), rhs)
    }
}

// Q * u128 -> Q
impl<'id, T> Mul<u128> for Q<'id, T>
where
    T: Copy + One + Zero + PartialEq + From<u128> + Default,
{
    type Output = Q<'id, T>;
    #[inline]
    fn mul(self, rhs: u128) -> Self::Output {
        t_mul_q(rhs.into(), self)
    }
}

// u128 + Q -> Q
impl<'id, T> Add<Q<'id, T>> for u128
where
    T: Copy + One + Zero + PartialEq + From<u128> + Default,
{
    type Output = Q<'id, T>;
    #[inline]
    fn add(self, rhs: Q<'id, T>) -> Self::Output {
        t_add_q(self.into(), rhs)
    }
}

// L + u128 -> L
impl<'id, T> Add<u128> for Q<'id, T>
where
    T: Copy + One + Zero + PartialEq + From<u128> + Default,
{
    type Output = Q<'id, T>;
    #[inline]
    fn add(self, rhs: u128) -> Self::Output {
        t_add_q(rhs.into(), self)
    }
}

#[derive(Clone, Copy)]
pub struct C<T>(pub T); // to avoid orphan rules

// C * L -> L
impl<'id, T> Mul<L<'id, T>> for C<T>
where
    T: Copy + One + Zero + PartialEq + From<u128> + Default,
{
    type Output = L<'id, T>;
    #[inline]
    fn mul(self, rhs: L<'id, T>) -> Self::Output {
        t_mul_l(self.0, rhs)
    }
}

// L * C -> L
impl<'id, T> Mul<C<T>> for L<'id, T>
where
    T: Copy + One + Zero + PartialEq + From<u128> + Default,
{
    type Output = L<'id, T>;
    #[inline]
    fn mul(self, rhs: C<T>) -> Self::Output {
        t_mul_l(rhs.0, self)
    }
}

// C + L -> L
impl<'id, T> Add<L<'id, T>> for C<T>
where
    T: Copy + One + Zero + PartialEq + From<u128> + Default,
{
    type Output = L<'id, T>;
    #[inline]
    fn add(self, rhs: L<'id, T>) -> Self::Output {
        t_add_l(self.0, rhs)
    }
}

// L + C -> L
impl<'id, T> Add<C<T>> for L<'id, T>
where
    T: Copy + One + Zero + PartialEq + From<u128> + Default,
{
    type Output = L<'id, T>;
    #[inline]
    fn add(self, rhs: C<T>) -> Self::Output {
        t_add_l(rhs.0, self)
    }
}

// C * Q -> Q
impl<'id, T> Mul<Q<'id, T>> for C<T>
where
    T: Copy + One + Zero + PartialEq + From<u128> + Default,
{
    type Output = Q<'id, T>;
    #[inline]
    fn mul(self, rhs: Q<'id, T>) -> Self::Output {
        t_mul_q(self.0, rhs)
    }
}

// Q * C -> Q
impl<'id, T> Mul<C<T>> for Q<'id, T>
where
    T: Copy + One + Zero + PartialEq + From<u128> + Default,
{
    type Output = Q<'id, T>;
    #[inline]
    fn mul(self, rhs: C<T>) -> Self::Output {
        t_mul_q(rhs.0, self)
    }
}

// C + Q -> Q
impl<'id, T> Add<Q<'id, T>> for C<T>
where
    T: Copy + One + Zero + PartialEq + From<u128> + Default,
{
    type Output = Q<'id, T>;
    #[inline]
    fn add(self, rhs: Q<'id, T>) -> Self::Output {
        t_add_q(self.0, rhs)
    }
}

// Q + C -> Q
impl<'id, T> Add<C<T>> for Q<'id, T>
where
    T: Copy + One + Zero + PartialEq + From<u128> + Default,
{
    type Output = Q<'id, T>;
    #[inline]
    fn add(self, rhs: C<T>) -> Self::Output {
        t_add_q(rhs.0, self)
    }
}
