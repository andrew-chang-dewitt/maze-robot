pub trait Functor<A> {
    type FHigherSelf<T>: Functor<T>;

    fn fmap<B>(self, f: impl Fn(A) -> B) -> Self::FHigherSelf<B>;
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
