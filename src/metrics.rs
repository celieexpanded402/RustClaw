use actix_web::{web, HttpResponse, get};

#[get("/metrics")]
async fn metrics() -> HttpResponse {
    let metrics = "# TYPE http_requests_total counter\nhttp_requests_total 42";
    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4; charset=utf-8")
        .body(metrics)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/metrics").service(metrics));
}