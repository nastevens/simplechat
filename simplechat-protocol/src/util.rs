use crate::Error;

/// Convenience trait to easily convert errors to `Error::InvalidFrame`
pub trait ResultExt<T> {
    fn or_invalid_frame(self) -> Result<T, Error>;
}

impl<T, E> ResultExt<T> for Result<T, E> {
    fn or_invalid_frame(self) -> Result<T, Error> {
        self.or(Err(Error::InvalidFrame))
    }
}

impl<T> ResultExt<T> for Option<T> {
    fn or_invalid_frame(self) -> Result<T, Error> {
        self.ok_or(Error::InvalidFrame)
    }
}
