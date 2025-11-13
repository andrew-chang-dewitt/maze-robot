mod applicative;
mod functor;
mod monad;
mod state;

pub use applicative::Applicative;
pub use functor::Functor;
pub use monad::{Monad, MonadTrans};
pub use state::State;

pub fn compose<'a, T, U, V, P, Q>(p: P, q: Q) -> impl Fn(T) -> V
where
    P: 'a + Fn(T) -> U,
    Q: 'a + Fn(U) -> V,
{
    move |z| q(p(z))
}

pub fn apply<'a, T, U, F: 'a + Fn(&'a T) -> U>(x: &'a T) -> impl Fn(F) -> U {
    move |f: F| f(x)
}
