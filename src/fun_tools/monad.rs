use super::Applicative;

pub trait Monad<'a, A>: Applicative<'a, A>
where
    A: 'a,
    Self: 'a,
{
    type MHigherType<T: 'a>: Monad<'a, T>;

    fn bind<B, F: Fn(A) -> Self::MHigherType<B>>(self, f: F) -> Self::MHigherType<B>;

    fn seq<B>(self, next: Self::MHigherType<B>) -> Self::MHigherType<B>;
}

#[cfg(test)]
mod tests {
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
}
