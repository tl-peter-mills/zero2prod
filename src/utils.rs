use actix_web::error::InternalError;
use actix_web::http::header::LOCATION;
use actix_web::HttpResponse;

pub fn e400<T: std::fmt::Debug + std::fmt::Display>(e: T) -> actix_web::Error
where
    T: std::fmt::Debug + std::fmt::Display + 'static
{
    actix_web::error::ErrorBadRequest(e)
}

pub fn e500<T>(e: T) -> InternalError<T> {
    InternalError::from_response(e, HttpResponse::InternalServerError().finish())
}

pub fn see_login() -> HttpResponse {
    see_other("/login")
}

pub fn see_other(location: &str) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((LOCATION, location))
        .finish()
}
