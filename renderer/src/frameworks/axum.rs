#[cfg(feature = "with-axum")]
pub use axum_core::response::{IntoResponse, Response};
#[cfg(feature = "with-axum-06")]
pub use axum_core_03::response::{IntoResponse, Response};
#[cfg(feature = "with-axum")]
use http::StatusCode;
#[cfg(feature = "with-axum-06")]
use http_02 as http;
#[cfg(feature = "with-axum-06")]
use http_02::StatusCode;

use crate::{RenderContext, Renderer};

impl Renderer {
    pub fn render_response<T: RenderContext>(&self, data: &T) -> Response {
        match data.render(self) {
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
