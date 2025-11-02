use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug)]
pub struct HopperError(pub anyhow::Error);

impl<E> From<E> for HopperError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

impl IntoResponse for HopperError {
    fn into_response(self) -> Response {
        {
            tracing::error!(error = ?self.0, "internal server error");
            (StatusCode::INTERNAL_SERVER_ERROR).into_response()
        }
    }
}
