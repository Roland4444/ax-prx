use axum::{
    extract::Request,
    http::StatusCode,
    middleware::{self, Next},
    response::Response,
    Router,
};
use axum_reverse_proxy::ReverseProxy;

const GLPI_UPSTREAM: &str = "https://glpi.upshepard.ru";
const GLPI_PATH: &str = "/glpi";
const PORT: u16 = 11112;

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
    // Прокси слушает путь /glpi и отправляет запросы на GLPI_UPSTREAM
    let proxy = ReverseProxy::new("/glpi", GLPI_UPSTREAM);
    let proxy_router: Router = proxy.into();
    proxy_router.layer(middleware::from_fn(auth_middleware))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = create_glpi_proxy();

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", PORT)).await?;
    println!("🚀 GLPI прокси запущен на http://localhost:{}", PORT);
    println!("➡️  Доступен по http://localhost:{}{}/ (с заголовком REMOTE_USER)", PORT, GLPI_PATH);

    axum::serve(listener, app).await?;
    Ok(())
}