use axum::{
    extract::Request,
    http::StatusCode,
    middleware::{self, Next},
    response::Response,
    routing::get,
    Router,
};
use axum_reverse_proxy::ReverseProxy;
use reqwest::Client;
use std::sync::Arc;

const GLPI_UPSTREAM: &str = "https://glpi.upshepard.ru";
const GLPI_PATH: &str = "/glpi";
const PORT: u16 = 11112;

async fn lisp_app_handler() -> &'static str {
    "Это ваше приложение на Lisp (заглушка)."
}

async fn auth_middleware(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let user = req
        .headers()
        .get("x-user-id")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("post_only")
        .to_string();

    req.headers_mut().insert(
        "REMOTE_USER",
        user.parse().map_err(|_| StatusCode::BAD_REQUEST)?,
    );

    Ok(next.run(req).await)
}

fn create_glpi_proxy() -> Router {
    // Создаём HTTP-клиент с игнорированием ошибок SSL
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .expect("Failed to create HTTP client");

    // Базовый путь "/" означает, что прокси принимает пути, начинающиеся с "/"
    let proxy = ReverseProxy::new("/", GLPI_UPSTREAM).with_client(Arc::new(client));
    let proxy_router: Router = proxy.into();
    proxy_router.layer(middleware::from_fn(auth_middleware))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = Router::new()
        .route("/", get(lisp_app_handler))
        .route("/chat", get(lisp_app_handler))
        .nest(GLPI_PATH, create_glpi_proxy());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", PORT)).await?;
    println!("🚀 Сервер запущен на http://localhost:{}", PORT);
    println!("➡️  Основное приложение (заглушка): http://localhost:{}/", PORT);
    println!("➡️  GLPI через прокси: http://localhost:{}{}/ (с заголовком REMOTE_USER)", PORT, GLPI_PATH);

    axum::serve(listener, app).await?;
    Ok(())
}