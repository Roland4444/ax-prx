use axum::{
    extract::Request,
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use axum_reverse_proxy::ReverseProxy;
use std::collections::HashMap;
use tower_http::validate_request::ValidateRequestHeaderLayer;

const GLPI_UPSTREAM: &str = "https://glpi.upshepard.ru";
const GLPI_PATH: &str = "/glpi"; 

// --- Наше приложение (то, что сейчас на Hunchentoot) ---
async fn lisp_app_handler() -> &'static str {
    "Это ваше приложение на Lisp (заглушка). Тут будет /chat и всё остальное."
}

#[derive(Clone)]
struct AuthState {
}

async fn auth_middleware<B>(mut req: Request<B>, next: Next<B>) -> Result<Response, StatusCode> {
    let user = req
        .headers()
        .get("x-user-id")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("post_only");

    req.headers_mut().insert(
        "REMOTE_USER",
        user.parse().unwrap()
    );

    Ok(next.run(req).await)
}

fn create_glpi_proxy() -> Router {
    let proxy = ReverseProxy::new(GLPI_PATH, GLPI_UPSTREAM);

    let proxy_router: Router = proxy.into();

    proxy_router.layer(middleware::from_fn(auth_middleware))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = Router::new()
        .route("/", get(lisp_app_handler))
        .route("/chat", get(lisp_app_handler))
        .merge(create_glpi_proxy());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:11111").await?;
    println!("🚀 Сервер запущен на http://localhost:11111");
    println!("➡️  Ваше приложение: http://localhost:11111/chat");
    println!("➡️  GLPI через прокси: http://localhost:11111/glpi/ (с заголовком REMOTE_USER)");

    axum::serve(listener, app).await?;
    Ok(())
}