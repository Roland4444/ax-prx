use axum::{
    extract::Request,
    http::StatusCode,
    middleware::{self, Next},
    response::Response,
    routing::get,               // <-- добавлен импорт для get
    Router,
};
use axum_reverse_proxy::ReverseProxy;
// import можно удалить, если не используется; пока оставим закомментированным
// use tower_http::validate_request::ValidateRequestHeaderLayer;

const GLPI_UPSTREAM: &str = "https://glpi.upshepard.ru";
const GLPI_PATH: &str = "/glpi";

async fn lisp_app_handler() -> &'static str {
    "Это ваше приложение на Lisp (заглушка)."
}

async fn auth_middleware(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    // Извлекаем идентификатор пользователя (в реальности из куки или токена)
    let user = req
        .headers()
        .get("x-user-id")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("post_only")
        .to_string(); // копируем, чтобы не держать ссылку на req

    // Добавляем заголовок REMOTE_USER для GLPI
    req.headers_mut().insert(
        "REMOTE_USER",
        user.parse().map_err(|_| StatusCode::BAD_REQUEST)?,
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