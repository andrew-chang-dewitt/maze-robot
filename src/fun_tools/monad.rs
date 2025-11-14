use super::Functor;

pub fn ret<T, M: Monad<Unwrapped = T>>(val: T) -> M {
    <M as Monad>::ret(val)
}

pub trait Monad: Functor {
    fn bind<B, F: FnMut(Self::Unwrapped) -> Self::Wrapped<B>>(self, f: F) -> Self::Wrapped<B>;

    fn seq<B>(self, next: Self::Wrapped<B>) -> Self::Wrapped<B>;

    fn ret(val: Self::Unwrapped) -> Self;
}

#[cfg(test)]
mod tests {
    use proptest::{prop_assert_eq, proptest};
    use rstest::rstest;

    use super::*;

    impl<A> Monad for Option<A> {
        fn bind<B, F: FnMut(A) -> Self::Wrapped<B>>(self, f: F) -> Self::Wrapped<B> {
            self.and_then(f)
        }

        fn seq<B>(self, next: Self::Wrapped<B>) -> Self::Wrapped<B> {
            self.and(next)
        }

        fn ret(val: A) -> Self {
            Some(val)
        }
    }

    #[rstest]
    #[case(Some(5), Some(String::from("5")))]
    #[case(None, None)]
    fn bind_transforms_state_in_context(#[case] a: Option<isize>, #[case] exp: Option<String>) {
        let f = |x| Some(format!("{x}"));

        let b = a.bind(f);
        assert_eq!(b, exp)
    }

    proptest! {
        // -- Monad laws
        //
        //   1. Left Identity:
        //
        //        return x >>= f = f x
        //
        //      or equivalently:
        //
        //        do { y <- return x; f y } = do { f x }
        #[test]
        fn left_identity(i in isize::MIN..isize::MAX) {
            fn f(x: isize) -> Option<isize> {
                if x > 0 { Some(x / 2) } else { None }
            }

            let left = ret::<isize, Option<isize>>(i).bind(f);
            let right = f(i);
            prop_assert_eq!(left, right);
        }

        //   2. Right Identity:
        //
        //        m >>= return = m
        //
        //      or equivalently:
        //
        //        do { x <- m; return x } = do { m }
        #[test]
        fn right_identity(i in isize::MIN..isize::MAX) {
            let left = Some(i).bind(|x| ret(x));
            let right = Some(i);
            prop_assert_eq!(left, right);
        }

        //   3. Associativity:
        //
        //        (m >>= \x -> f x) >>= g = m >>= (\x -> f x >>= g)
        //
        //      or equivalently:
        //
        //        do                     do
        //          y <- do x <- m   =     x <- m
        //                  f x            y <- f x
        //          g y                    g y
        #[test]
        fn associativity(i in isize::MIN..isize::MAX) {
            fn f(x: isize) -> Option<isize> {
                if x > 0 { Some(x / 2) } else { None }
            }

            fn g(x: isize) -> Option<isize> {
                Some(x * 2)
            }

            let left = (Some(i).bind(|x| f(x))).bind(g);
            let right = Some(i).bind(f).bind(g);
            prop_assert_eq!(left, right);
        }
    }
}
