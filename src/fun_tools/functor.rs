pub trait Functor<A> {
    type HigherSelf<T>: Functor<T>;

    fn map<B>(self, f: impl Fn(A) -> B) -> Self::HigherSelf<B>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_single_content_using_fn() {
        #[derive(Debug, PartialEq, Eq)]
        struct Thing<T>(T);

        impl<A> Functor<A> for Thing<A> {
            type HigherSelf<T> = Thing<T>;

            fn map<B>(self, f: impl Fn(A) -> B) -> Self::HigherSelf<B> {
                Thing(f(self.0))
            }
        }

        let a = Thing(1);
        let b = a.map(|x| format!("{x}"));

        assert_eq!(b, Thing("1".to_string()))
    }

    #[test]
    fn maps_listlike_contents_using_fn() {
        struct List<T>(Vec<T>);

        impl<T> List<T> {
            pub fn new(vec: Vec<T>) -> Self {
                Self(vec)
            }
        }

        impl<A> Functor<A> for List<A> {
            type HigherSelf<T> = List<T>;

            fn map<B>(self, f: impl Fn(A) -> B) -> Self::HigherSelf<B> {
                List(self.0.into_iter().map(f).collect())
            }
        }

        let a = List::new(vec![1, 2, 3, 4, 5]);
        let b = a.map(|x| format!("{x}"));

        let act: Vec<String> = b.0;
        let exp = vec![
            String::from("1"),
            String::from("2"),
            String::from("3"),
            String::from("4"),
            String::from("5"),
        ];

        assert_eq!(act, exp)
    }
}
