pub enum OptionalStatic<T: 'static> {
    Static(&'static T),
    Owned(T),
}

impl<T> OptionalStatic<T> {
    pub fn get_ref(&self) -> &T {
        match self {
            OptionalStatic::Static(value) => *value,
            OptionalStatic::Owned(value) => value,
        }
    }
}