pub use axum_core::response::{IntoResponse, Response};
use http::StatusCode;

use crate::{RenderContext, Renderer};

impl Renderer {
    pub fn into_response<T: RenderContext>(&self, context: &T) -> Response {
        match self.render(context) {
            Ok(body) => {
                let headers = [(
                    http::header::CONTENT_TYPE,
                    http::HeaderValue::from_static(T::MIME_TYPE),
                )];

                (headers, body).into_response()
            }
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
