use actix_web::{HttpResponse, http::header::LOCATION, web};
use secrecy::Secret;

#[derive(serde::Deserialize)]
pub struct FormData {
    _username: String,
    _password: Secret<String>,
}

pub async fn login(_form: web::Form<FormData>) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((LOCATION, "/"))
        .finish()
}
