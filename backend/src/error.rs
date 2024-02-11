use rocket::serde::json::Json;
use rocket::{http::Status, response::status};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct ErrorResponse {
    message: String,
}

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum Error {
    GeneralError(String),
}

impl<'r> rocket::response::Responder<'r, 'r> for Error {
    fn respond_to(self, request: &rocket::Request) -> rocket::response::Result<'r> {
        match self {
            Error::GeneralError(message) => {
                let error_response = Json(ErrorResponse { message });
                status::Custom(Status::InternalServerError, error_response).respond_to(request)
            }
        }
    }
}
