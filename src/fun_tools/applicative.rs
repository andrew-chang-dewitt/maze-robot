use super::Functor;

pub fn pure<'a, T: 'a, A: Applicative<'a, T>>(val: T) -> A {
    <A as Applicative<'a, T>>::pure(val)
}

pub trait Applicative<'a, A>: Functor<A>
where
    A: 'a,
    Self: 'a,
{
    type AHigherSelf<T: 'a>: Applicative<'a, T>;

    fn pure(val: A) -> Self;

    fn apply<B, F: Fn(&'a A) -> B>(&'a self, fs: Self::AHigherSelf<F>) -> Self::AHigherSelf<B>;
}

#[cfg(test)]
mod tests {
    use super::*;

    impl<'a, A> Applicative<'a, A> for Option<A>
    where
        Self: 'a,
    {
        type AHigherSelf<T: 'a> = Option<T>;

        fn pure(val: A) -> Self {
            Some(val)
        }

        fn apply<B: 'a, F: 'a + Fn(&'a A) -> B>(
            &'a self,
            fs: Self::AHigherSelf<F>,
        ) -> Self::AHigherSelf<B> {
            match (self, fs) {
                (Some(x), Some(f)) => Some(f(x)),
                _ => None,
            }
        }
    }

    #[test]
    fn apply_container_to_another() {
        let a: Option<usize> = pure(5);
        let f = pure(|x| format!("{x}"));

        let b = a.apply(f);
        assert_eq!(b, Some(String::from("5")))
    }

    impl<'a, A> Applicative<'a, A> for Vec<A>
    where
        Self: 'a,
    {
        type AHigherSelf<T: 'a> = Vec<T>;

        fn pure(val: A) -> Self {
            vec![val]
        }

        /// Applies every function in `fs` to every value in `Self`.
        fn apply<B: 'a, F: 'a + Fn(&'a A) -> B>(
            &'a self,
            fs: Self::AHigherSelf<F>,
        ) -> Self::AHigherSelf<B> {
            self.iter().flat_map(|x| fs.iter().map(|f| f(x))).collect()
        }
    }

    #[test]
    fn apply_vec_to_another() {
        let a: Vec<usize> = vec![5, 10, 15, 20];
        let f = vec![|x| format!("{}", x * 2), |_| String::from("hello!")];

        let b = a.apply(f);
        assert_eq!(
            b,
            vec![
                String::from("10"),
                String::from("hello!"),
                String::from("20"),
                String::from("hello!"),
                String::from("30"),
                String::from("hello!"),
                String::from("40"),
                String::from("hello!")
            ]
        )
    }
}
