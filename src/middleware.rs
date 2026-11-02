use axum::http::{HeaderName, HeaderValue};
use axum::{
    Router,
    extract::{Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use tower_http::set_header::SetResponseHeaderLayer;

const MIN_SECRET_LEN: usize = 16;

fn header_layer(name: &'static str, value: &'static str) -> SetResponseHeaderLayer<HeaderValue> {
    SetResponseHeaderLayer::overriding(
        HeaderName::from_static(name),
        HeaderValue::from_static(value),
    )
}

#[derive(Clone)]
struct OriginLock {
    secrets: Arc<Vec<Vec<u8>>>,
}

/// Comma-separated so a new secret can be added before the old one is removed.
/// Panics at startup on any entry shorter than MIN_SECRET_LEN.
pub fn parse_secrets(raw: &str) -> Vec<Vec<u8>> {
    let secrets: Vec<Vec<u8>> = raw
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.as_bytes().to_vec())
        .collect();
    for s in &secrets {
        assert!(
            s.len() >= MIN_SECRET_LEN,
            "ORIGIN_SECRET entries must be at least {MIN_SECRET_LEN} bytes"
        );
    }
    secrets
}

fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false; // length leaks, content does not
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// Empty secrets leaves the router unlocked (local dev). main.rs decides
/// whether that is allowed and should panic on Fly when the secret is missing.
pub fn lock_origin(router: Router, secrets: Vec<Vec<u8>>) -> Router {
    if secrets.is_empty() {
        return router;
    }
    let lock = OriginLock {
        secrets: Arc::new(secrets),
    };
    router.layer(middleware::from_fn_with_state(lock, check_origin))
}

async fn check_origin(State(lock): State<OriginLock>, req: Request, next: Next) -> Response {
    if req.uri().path() == "/health" {
        return next.run(req).await;
    }
    let ok = req
        .headers()
        .get("x-origin-verify")
        .map(|v| v.as_bytes())
        // fold with `|`, not any(), so every secret is compared
        .is_some_and(|v| lock.secrets.iter().fold(false, |acc, s| acc | ct_eq(s, v)));
    if ok {
        next.run(req).await
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

/// Call as apply_security_headers(lock_origin(router, secrets)) so the
/// header layers sit outside the lock and its 404 carries them.
pub fn apply_security_headers(router: Router) -> Router {
    let csp = "default-src 'self'; \
        script-src 'self'; \
        style-src 'self' https://fonts.googleapis.com; \
        font-src 'self' https://fonts.gstatic.com; \
        img-src 'self' data:; \
        connect-src 'self'; \
        frame-ancestors 'none'; \
        base-uri 'self'; \
        form-action 'self'";

    router
        .layer(header_layer("content-security-policy", csp))
        .layer(header_layer(
            "strict-transport-security",
            "max-age=63072000; includeSubDomains; preload",
        ))
        .layer(header_layer("x-content-type-options", "nosniff"))
        .layer(header_layer("x-frame-options", "DENY"))
        .layer(header_layer("x-xss-protection", "0"))
        .layer(header_layer(
            "referrer-policy",
            "strict-origin-when-cross-origin",
        ))
        .layer(header_layer(
            "permissions-policy",
            "camera=(), microphone=(), geolocation=(), payment=()",
        ))
        .layer(header_layer("cross-origin-opener-policy", "same-origin"))
        .layer(header_layer("cross-origin-resource-policy", "same-origin"))
        .layer(header_layer("x-permitted-cross-domain-policies", "none"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::routing::get;
    use tower::ServiceExt;

    const SECRET_A: &str = "0123456789abcdef";
    const SECRET_B: &str = "fedcba9876543210";

    fn router(secrets: &str) -> Router {
        apply_security_headers(lock_origin(
            Router::new()
                .route("/", get(|| async { "ok" }))
                .route("/health", get(|| async { "ok" })),
            parse_secrets(secrets),
        ))
    }

    fn req(path: &str, secret: Option<&str>) -> Request<Body> {
        let mut b = Request::builder().uri(path);
        if let Some(s) = secret {
            b = b.header("x-origin-verify", s);
        }
        b.body(Body::empty()).unwrap()
    }

    #[tokio::test]
    async fn applies_all_expected_security_headers() {
        let response = router("").oneshot(req("/", None)).await.unwrap();

        let headers = response.headers();
        assert!(headers.get("content-security-policy").is_some());
        assert_eq!(
            headers.get("strict-transport-security").unwrap(),
            "max-age=63072000; includeSubDomains; preload"
        );
        assert_eq!(headers.get("x-content-type-options").unwrap(), "nosniff");
        assert_eq!(headers.get("x-frame-options").unwrap(), "DENY");
        assert_eq!(headers.get("x-xss-protection").unwrap(), "0");
        assert_eq!(
            headers.get("referrer-policy").unwrap(),
            "strict-origin-when-cross-origin"
        );
        assert_eq!(
            headers.get("permissions-policy").unwrap(),
            "camera=(), microphone=(), geolocation=(), payment=()"
        );
        assert_eq!(
            headers.get("cross-origin-opener-policy").unwrap(),
            "same-origin"
        );
        assert_eq!(
            headers.get("cross-origin-resource-policy").unwrap(),
            "same-origin"
        );
        assert_eq!(
            headers.get("x-permitted-cross-domain-policies").unwrap(),
            "none"
        );
    }

    #[tokio::test]
    async fn lock_rejection_carries_all_security_headers() {
        let app = router(SECRET_A);

        for r in [req("/", None), req("/", Some("wrong-secret-value"))] {
            let res = app.clone().oneshot(r).await.unwrap();
            assert_eq!(res.status(), StatusCode::NOT_FOUND);

            let h = res.headers();
            assert_eq!(h.get("x-content-type-options").unwrap(), "nosniff");
            assert_eq!(h.get("x-frame-options").unwrap(), "DENY");
            assert!(h.get("content-security-policy").is_some());
            assert!(h.get("strict-transport-security").is_some());
        }
    }

    #[tokio::test]
    async fn correct_secret_is_allowed() {
        let res = router(SECRET_A)
            .oneshot(req("/", Some(SECRET_A)))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn health_is_open_without_secret() {
        let res = router(SECRET_A)
            .oneshot(req("/health", None))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn rotation_accepts_old_and_new_secret() {
        let app = router(&format!("{SECRET_A},{SECRET_B}"));
        for s in [SECRET_A, SECRET_B] {
            let res = app.clone().oneshot(req("/", Some(s))).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }
        let res = app
            .oneshot(req("/", Some("not-either-secret")))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn no_secrets_means_unlocked() {
        let res = router("").oneshot(req("/", None)).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[test]
    #[should_panic(expected = "at least")]
    fn short_secret_panics() {
        parse_secrets("short");
    }
}
