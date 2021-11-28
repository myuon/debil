use failure::Fail;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "not_found")]
    NotFoundError,
    #[fail(display = "mysql_error")]
    MySQLError(#[cause] mysql_async::Error),
}

impl From<mysql_async::Error> for Error {
    fn from(err: mysql_async::Error) -> Error {
        Error::MySQLError(err)
    }
}
