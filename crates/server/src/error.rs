use dropshot::HttpError;

/// Map any database error to a 500 "Internal Server Error"
impl From<super::database::Error> for HttpError {
    fn from(val: super::database::Error) -> Self {
        HttpError::for_internal_error(val.to_string())
    }
}
