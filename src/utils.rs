pub trait ResultExt<T, E> {
    fn replace_err<Other>(self, other: Other) -> Result<T, Other>;
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
    fn replace_err<Other>(self, other: Other) -> Result<T, Other> {
        self.or(Err(other))
    }
}
