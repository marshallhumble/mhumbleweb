use axum::Router;
use axum::http::{HeaderName, HeaderValue};
use tower_http::set_header::SetResponseHeaderLayer;

fn header_layer(name: &'static str, value: &'static str) -> SetResponseHeaderLayer<HeaderValue> {
    SetResponseHeaderLayer::overriding(
        HeaderName::from_static(name),
        HeaderValue::from_static(value),
    )
}

pub fn apply_security_headers(router: Router) -> Router {
    let csp = "default-src 'self'; \
        script-src 'self'; \
        style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; \
        font-src 'self' https://fonts.gstatic.com; \
        img-src 'self' data:; \
        connect-src 'self'; \
        frame-ancestors 'none'; \
        base-uri 'self'; \
        form-action 'self'";

    router
        .layer(header_layer("content-security-policy", csp))
        .layer(header_layer("strict-transport-security", "max-age=63072000; includeSubDomains; preload"))
        .layer(header_layer("x-content-type-options", "nosniff"))
        .layer(header_layer("x-frame-options", "DENY"))
        .layer(header_layer("x-xss-protection", "0"))
        .layer(header_layer("referrer-policy", "strict-origin-when-cross-origin"))
        .layer(header_layer("permissions-policy", "camera=(), microphone=(), geolocation=(), payment=()"))
}