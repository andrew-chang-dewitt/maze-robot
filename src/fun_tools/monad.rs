use super::Applicative;

pub fn ret<'a, T: 'a, M: Monad<'a, T>>(val: T) -> M {
    <M as Monad<'a, T>>::ret(val)
}

pub trait Monad<'a, A: 'a>: Applicative<'a, A> {
    type MHigherType<T: 'a>: Monad<'a, T>;

    fn bind<B, F: Fn(A) -> Self::MHigherType<B>>(self, f: F) -> Self::MHigherType<B>;

    fn seq<B>(self, next: Self::MHigherType<B>) -> Self::MHigherType<B>;

    fn ret(val: A) -> Self
    where
        Self: Sized,
    {
        Self::pure(val)
    }
}

#[cfg(test)]
mod tests {
    use proptest::{prop_assert_eq, proptest};
    use rstest::rstest;

    use super::*;

    impl<'a, A> Monad<'a, A> for Option<A>
    where
        A: 'a,
        Self: 'a,
    {
        type MHigherType<T: 'a> = Option<T>;

        fn bind<B: 'a, F: Fn(A) -> Self::MHigherType<B>>(self, f: F) -> Self::MHigherType<B> {
            self.and_then(f)
        }

        fn seq<B: 'a>(self, next: Self::MHigherType<B>) -> Self::MHigherType<B> {
            self.and(next)
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
