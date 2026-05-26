use axum::{extract::Path, response::{Html, Response}, body::Body};
use axum::http::{header, StatusCode};

const INDEX_HTML: &str = include_str!("../static/index.html");
const STYLE_CSS:  &str = include_str!("../static/style.css");

pub async fn index() -> Html<&'static str> { Html(INDEX_HTML) }

pub async fn static_file(Path(file): Path<String>) -> Response<Body> {
    match file.as_str() {
        "style.css" => Response::builder()
            .header(header::CONTENT_TYPE, "text/css")
            .body(Body::from(STYLE_CSS))
            .unwrap_or_default(),
        _ => Response::builder().status(StatusCode::NOT_FOUND).body(Body::empty()).unwrap_or_default(),
    }
}