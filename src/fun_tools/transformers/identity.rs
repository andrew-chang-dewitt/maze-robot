use super::super::{Functor, Monad};
use super::MonadTrans;

pub struct IdentityT<M>(M);

impl<M: Functor> Functor for IdentityT<M> {
    type Unwrapped = M::Unwrapped;
    type Wrapped<T> = IdentityT<M::Wrapped<T>>;

    fn fmap<B>(self, f: impl FnMut(Self::Unwrapped) -> B) -> Self::Wrapped<B> {
        IdentityT(self.0.fmap(f))
    }
}

impl<M: Monad> Monad for IdentityT<M> {
    fn ret(val: Self::Unwrapped) -> Self {
        IdentityT(<M as Monad>::ret(val))
    }

    fn bind<B, F: FnMut(Self::Unwrapped) -> Self::Wrapped<B>>(self, mut f: F) -> Self::Wrapped<B> {
        IdentityT(self.0.bind(|x| f(x).0))
    }

    fn seq<B>(self, next: Self::Wrapped<B>) -> Self::Wrapped<B> {
        IdentityT(self.0.seq(next.0))
    }
}

impl<M: Monad> MonadTrans for IdentityT<M> {
    type Base = M;

    fn lift(base: Self::Base) -> Self {
        IdentityT(base)
    }
}

impl<T: Clone> Clone for IdentityT<T> {
    fn clone(&self) -> Self {
        IdentityT(self.0.clone())
    }
}

impl<T: Copy> Copy for IdentityT<T> {}

#[cfg(test)]
mod tests {
    pub use super::*;

    #[test]
    fn is_equivalent_to_contained_monad() {
        let a = IdentityT(Some(42));
        let b = Some(42);
        let c: IdentityT<Option<isize>> = IdentityT(None);
        let d: Option<isize> = None;

        let f = |x| format!("{x}");
        let ga = |x| IdentityT(Some(format!("{x}")));
        let gb = |x| Some(format!("{x}"));

        assert_eq!(a.fmap(f).0, b.fmap(f));
        assert_eq!(c.fmap(f).0, d.fmap(f));
        assert_eq!(a.bind(ga).0, b.bind(gb));
        assert_eq!(c.bind(ga).0, d.bind(gb));
    }
}
