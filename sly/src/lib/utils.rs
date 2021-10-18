use candid::Principal;

/// Flatten a result, because .flatten() is unstable.
#[inline]
pub fn result_flatten<T, E>(result: Result<Result<T, E>, E>) -> Result<T, E> {
    match result {
        Ok(Ok(r)) => Ok(r),
        Ok(Err(e)) => Err(e),
        Err(e) => Err(e),
    }
}

/// Checks if the provided string is a valid principal id.
#[inline]
#[allow(dead_code)]
fn is_principal(text: &str) -> Result<(), String> {
    Principal::from_text(text)
        .map(|_| ())
        .map_err(|_| "Not a valid principal id.".to_string())
}
