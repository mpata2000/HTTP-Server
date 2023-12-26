use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub enum HttpStatus {
    Ok,
    Created,
    NoContent,
    BadRequest,
    NotFound,
    Conflict,
    UnprocessableEntity,
    InternalServerError,
}

impl Display for HttpStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let code = match self {
            HttpStatus::Ok => "200 OK",
            HttpStatus::Created => "201 Created",
            HttpStatus::NoContent => "204 No Content",
            HttpStatus::BadRequest => "400 Bad Request",
            HttpStatus::NotFound => "404 Not Found",
            HttpStatus::Conflict => "409 Conflict",
            HttpStatus::UnprocessableEntity => "422 Unprocessable Entity",
            HttpStatus::InternalServerError => "500 Internal Server Error",
        };

        write!(f, "{}", code)
    }
}
