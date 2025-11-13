use std::marker::PhantomData;

use super::{Functor, Monad};

#[derive(Clone, Copy)]
pub struct State<F, M, A, S>
where
    F: Fn(S) -> M,
{
    _f: F,
    _m: PhantomData<M>,
    _a: PhantomData<A>,
    _s: PhantomData<S>,
}

impl<F, M, A, S> State<F, M, A, S>
where
    F: Fn(S) -> M,
{
    pub fn new(f: F) -> Self {
        Self {
            _f: f,
            _m: PhantomData,
            _a: PhantomData,
            _s: PhantomData,
        }
    }

    pub fn run(self, s: S) -> M {
        (self._f)(s)
    }
}

impl<F, M, A, S, N, G> Functor<A> for State<F, M, A, S>
where
    F: Fn(S) -> M,
    M: Monad<(A, S)>,
{
    type FHigherSelf<B>
        = State<G, N, B, S>
    where
        N: Monad<(B, S)>,
        G: Fn(S) -> N;

    fn fmap<B>(self, f: impl Fn(A) -> B) -> Self::FHigherSelf<B> {
        State::new(|s1| self.run(s1).bind(|(a, s2)| (f(a), s2)))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use super::*;

    #[test]
    fn test_state_stack() {
        let pop: State<_, _, usize, _> =
            State::new(|mut s: VecDeque<usize>| s.pop_back().map(|a| (a, s)));
        let push = |x: usize| {
            State::<_, _, usize, _>::new(move |mut s: VecDeque<usize>| {
                s.push_back(x);
                Some(((), s))
            })
        };

        let s1 = VecDeque::new();

        let act = State::run(push(5), s1)
            .and_then(|(_, s2)| State::run(push(7), s2))
            .and_then(|(_, s3)| State::run(pop.clone(), s3))
            .and_then(|(x, s4)| State::run(pop, s4).and_then(|(y, _)| Some(x + y)));

        let exp = Some(12);

        assert_eq!(act, exp)
    }
}
