pub mod layouts;
pub mod views;

use axum::http::header;
use axum::response::{IntoResponse, Response};
use maud::Markup;

pub struct HtmlTemplate(pub Markup);

impl IntoResponse for HtmlTemplate {
    fn into_response(self) -> Response {
        let mut response = self.0.into_response();
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("text/html"),
        );
        response
    }
}
