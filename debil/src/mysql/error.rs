#[derive(Debug)]
pub enum Error {
    NotFoundError,
    MySQLError(mysql_async::Error),
}

impl From<mysql_async::Error> for Error {
    fn from(err: mysql_async::Error) -> Error {
        Error::MySQLError(err)
    }
}
