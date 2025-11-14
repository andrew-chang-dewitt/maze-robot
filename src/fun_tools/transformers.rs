mod identity;
mod state;

pub use identity::IdentityT;
pub use state::StateT;

use super::Monad;

/// A type that transforms a given Monad of A into itself, granting that Monad all the
/// functionality of itself. A Monad Transformer is itself required to be a Monad (of the
/// given Monad).
pub trait MonadTrans {
    type Base: Monad;

    fn lift(base: Self::Base) -> Self;
}

pub fn lift<M: Monad, T: MonadTrans<Base = M>>(monad: M) -> T {
    <T as MonadTrans>::lift(monad)
}
