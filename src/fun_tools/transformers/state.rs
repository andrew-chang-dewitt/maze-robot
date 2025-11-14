use std::marker::PhantomData;

use crate::fun_tools::{Functor, Monad};

pub struct StateT<M, A, S, F>
where
    F: FnMut(S) -> M,
{
    _run_fn: F,
    _m: PhantomData<M>,
    _a: PhantomData<A>,
    _s: PhantomData<S>,
}

impl<M, A, S, F> StateT<M, A, S, F>
where
    F: FnMut(S) -> M,
{
    pub fn new(run_fn: F) -> Self {
        Self {
            _run_fn: run_fn,
            _m: PhantomData,
            _a: PhantomData,
            _s: PhantomData,
        }
    }

    pub fn run(mut self, s: S) -> M {
        (self._run_fn)(s)
    }
}

pub fn map<M, A, S, F, N, B, G>(
    state: StateT<M, A, S, F>,
    f: impl FnMut(A) -> B,
) -> StateT<N, B, S, G>
where
    M: Monad<Unwrapped = (A, S)>,
    F: FnMut(S) -> M,
    N: Monad<Unwrapped = (B, S)>,
    // do i need to box this type and make it a dyn FnMut???
    G: FnMut(S) -> N,
{
    StateT::new(|s| state.run(s).fmap(|(a, s2)| (f(a), s2)))
}

// original haskell:
//
// instance (Functor m) => Functor (StateT s m) where
//     fmap f m = StateT $ \ s ->
//         fmap (\ ~(a, s') -> (f a, s')) $ runStateT m s
//
// translating to rust-like:
//
// instance (Functor m) => Functor (StateT s m) where
//     // finally, making this fn
//     // fmap :: StateT<m,a,s,(s -> m<(a,s)>)>
//             -> (a -> b)
//             -> StateT<m,b,s,(s -> m<(b,s)>)>
//     fmap(self, f) = StateT(        // and we're constructing a new StateT w/ it
//         |s|                        // which means this fn is :: s -> m<(b,s)>
//             // some monad of a & s
//             let res = runStateT(self, s)
//             res.fmap(
//                 // s_new because it was changed in the run_state op giving res
//                 |(a, s_new)| {    // and this fn is :: (a,s) -> (b,s)
//                     (f(a), s_new) // so f :: a -> b
//                 }
//             )
//             fmap (\ ~(a, s') -> (f a, s')) $ runStateT self s
//     )
// impl<M, A, S, F, V, N, G> Functor for StateT<M, A, S, F>
// where
//     M: Monad<Unwrapped = (A, S)>,
//     F: FnMut(S) -> M,
//     N: Monad<Unwrapped = (V, S)>,
//     G: FnMut(S) -> N,
// {
//     type Unwrapped = A;
//     type Wrapped<T> = StateT<N, T, S, G>;
//
//     fn fmap<B>(self, mut f: impl FnMut(Self::Unwrapped) -> B) -> Self::Wrapped<B> {
//         map(self, f)
//     }
// }
