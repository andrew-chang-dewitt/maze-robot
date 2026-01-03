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
