pub trait Functor<A> {
    type FHigherSelf<T>: Functor<T>;

    fn fmap<B>(self, f: impl Fn(A) -> B) -> Self::FHigherSelf<B>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    impl<A> Functor<A> for Option<A> {
        type FHigherSelf<T> = Option<T>;

        fn fmap<B>(self, f: impl Fn(A) -> B) -> Self::FHigherSelf<B> {
            self.map(f)
        }
    }

    #[test]
    fn maps_single_content_using_fn() {
        let a = Some(1);
        let b = a.map(|x| format!("{x}"));

        assert_eq!(b, Some("1".to_string()))
    }

    impl<A> Functor<A> for Vec<A> {
        type FHigherSelf<T> = Vec<T>;

        fn fmap<B>(self, f: impl Fn(A) -> B) -> Self::FHigherSelf<B> {
            self.into_iter().map(f).collect()
        }
    }

    #[test]
    fn maps_listlike_contents_using_fn() {
        let a = vec![1, 2, 3, 4, 5];
        let b = a.fmap(|x| format!("{x}"));

        let exp = vec![
            String::from("1"),
            String::from("2"),
            String::from("3"),
            String::from("4"),
            String::from("5"),
        ];

        assert_eq!(b, exp)
    }

    proptest! {
        // given:
        //   id :: a -> a
        //   id x = x
        // then:
        //   fmap id = id
        #[test]
        fn identity_law(i in isize::MIN..isize::MAX) {
            let a = Some(i);
            let b = None;
            let id = |x| x;

            prop_assert_eq!(a.fmap(id), Some(i));
            prop_assert_eq!(b.fmap(id), None);
        }

        // given:
        //   compose :: (a -> b) -> (b -> c) -> (a -> c)
        //   compose f g = \x -> g $ f x
        // then:
        //   fmap $ compose f g = compose (fmap f) (fmap g)
        //
        // testing w/
        //   f x = x * l,
        //   g y = y / 2,
        // s.t.
        //   compose f g = \x -> x * l / 2
        // where
        //   x * l <= MAX -> x <= MAX / l
        //   thus x < MAX / 100 -> l <= MAX / (MAX / 100) = 100
        // #[test]
        // fn composition_law(
        //     l in 1isize..100,
        //     i in (isize::MIN / 100)..(isize::MAX / 100),
        // ) {
        //     let a = Some(i);
        //     let f = |x| x * l;
        //     let g = |y| y / 2;

        //     fn compose<'a,T,U,V,P,Q>(p: P, q: Q ) -> impl Fn(T) -> V
        //     where
        //         P: 'a + Fn(T) -> U,
        //         Q: 'a + Fn(U) -> V,
        //     {
        //         move |z| q(p(z))
        //     };

        //     // TODO: how to actually compose to fmaps?
        //     prop_assert_eq!(a.fmap(compose(f,g)), Some(compose(f,g)(i) ))
        // }
    }
}
