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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Создаём прокси с базовым путём "/glpi" (он будет удаляться перед отправкой на апстрим)
    let proxy = ReverseProxy::new("/glpi", GLPI_UPSTREAM);
    // Превращаем прокси в роутер и добавляем middleware
    let proxy_router: Router = proxy.into();
    let proxy_with_auth = proxy_router.layer(middleware::from_fn(auth_middleware));

    // Монтируем прокси на путь /glpi
    let app = Router::new().nest(GLPI_PATH, proxy_with_auth);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", PORT)).await?;
    println!("🚀 GLPI прокси запущен на http://localhost:{}", PORT);
    println!("➡️  Доступен по http://localhost:{}{}/ (с заголовком REMOTE_USER)", PORT, GLPI_PATH);

    axum::serve(listener, app).await?;
    Ok(())
}