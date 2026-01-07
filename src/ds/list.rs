use nostd::boxed::Box;

#[derive(Debug)]
pub enum List<T> {
    Cons(T, Box<List<T>>),
    Nil,
}

impl<T> List<T> {
    pub fn empty() -> Self {
        Self::Nil
    }

    pub fn cons(t: T, l: Self) -> Self {
        List::Cons(t, Box::new(l))
    }

    pub fn one(t: T) -> Self {
        List::Cons(t, Box::new(List::Nil))
    }

    pub fn map<U>(self, _f: impl Fn(T) -> U) -> List<U> {
        todo!()
    }

    pub fn foldr<A>(self, init: A, f: impl Fn(A, T) -> A) -> A {
        match self {
            Self::Nil => init,
            Self::Cons(t, ts) => ts.foldr(f(init, t), f),
        }
    }
    // pub fn is_empty(&self) -> bool {
    //     match self {
    //         Self::Nil => true,
    //         _ => false,
    //     }
    // }
}

impl<T: PartialEq> PartialEq for List<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Cons(l, ls), Self::Cons(r, rs)) => {
                if l == r {
                    ls == rs
                } else {
                    false
                }
            }
            (Self::Nil, Self::Nil) => true,
            _ => false,
        }
    }
}
