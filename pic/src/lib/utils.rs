/// Flatten a result, because .flatten() is unstable.
#[inline]
pub fn result_flatten<T, E>(result: Result<Result<T, E>, E>) -> Result<T, E> {
    match result {
        Ok(Ok(r)) => Ok(r),
        Ok(Err(e)) => Err(e),
        Err(e) => Err(e),
    }
}
