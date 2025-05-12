use crate::{EscrowError, Result};

pub fn assert_err<T, E>(res: Result<T>, expected: E)
where
    E: std::fmt::Debug + PartialEq<E>,
    EscrowError: Into<E> + PartialEq<E>,
{
    match res {
        Err(e) => assert_eq!(e.into(), expected),
        Ok(_) => panic!("Expected error, got Ok"),
    }
}
