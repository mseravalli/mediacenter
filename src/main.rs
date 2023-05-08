use actix_files::Files;
use actix_web::{
    body::BoxBody,
    dev::ServiceResponse,
    error,
    http::{header::ContentType, StatusCode},
    middleware::{self, ErrorHandlerResponse, ErrorHandlers},
    web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder, Result,
};
use actix_web_lab::respond::Html;
use log::{debug, info};
use std::collections::HashMap;
use std::process::Command;
use tera::Tera;

// store tera template in application state
async fn index(
    tmpl: web::Data<tera::Tera>,
    query: web::Query<HashMap<String, String>>,
) -> Result<impl Responder, Error> {
    let s = if let Some(name) = query.get("name") {
        // submitted form
        let mut ctx = tera::Context::new();
        ctx.insert("name", name);
        ctx.insert("text", "Welcome!");
        tmpl.render("user.html", &ctx)
            .map_err(|_| error::ErrorInternalServerError("Template error"))?
    } else {
        tmpl.render("index.html", &tera::Context::new())
            .map_err(|_| error::ErrorInternalServerError("Template error"))?
    };

    Ok(Html(s))
}

fn create_vpn_server(region: &str) -> Result<String, String> {
    Command::new("/tmp/test.sh")
        .output()
        .expect("failed to execute process");
    Ok(format!("{region}"))
}

async fn connect(req: HttpRequest) -> HttpResponse {
    debug!("REQ: {req:?}");
    info!("REQ: {:?}", req.query_string());
    let query = web::Query::<HashMap<String, String>>::from_query(req.query_string())
        .map_err(|e| HttpResponse::BadRequest().json(format!("Wrong request format: {:?}", e)));

    let location = query.and_then(|x| match x.get("location") {
        Some(val) => Ok(val.clone()),
        None => Err(HttpResponse::BadRequest().json(format!("Location not present"))),
    });

    let res = location.and_then(|x| match x.as_str() {
        l if l == "italy" || l == "germany" => create_vpn_server(l)
            .map_err(|e| {
                HttpResponse::InternalServerError().json(format!(
                    "Error occurred while setting up the connection: {e}"
                ))
            })
            .map(|x| (l.to_string(), x)),
        l => Err(HttpResponse::BadRequest().json(format!("Cannot connect to {l}"))),
    });

    match res {
        Ok(x) => HttpResponse::Ok().json(format!("Started VPN setup to {}", x.0)),
        Err(e) => e,
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let port = 8080;
    info!("starting HTTP server at http://localhost:{}", port);

    HttpServer::new(|| {
        let tera = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();

        App::new()
            .app_data(web::Data::new(tera))
            .wrap(middleware::Logger::default())
            .service(Files::new("/img", "static/img/").show_files_listing())
            .service(Files::new("/src", "static/src/").show_files_listing())
            .service(web::resource("/connect").route(web::get().to(connect)))
            .service(web::resource("/").route(web::get().to(index)))
            .service(web::scope("").wrap(error_handlers()))
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}

// Custom error handlers, to return HTML responses when an error occurs.
fn error_handlers() -> ErrorHandlers<BoxBody> {
    ErrorHandlers::new().handler(StatusCode::NOT_FOUND, not_found)
}

// Error handler for a 404 Page not found error.
fn not_found<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<BoxBody>> {
    let response = get_error_response(&res, "Page not found");
    Ok(ErrorHandlerResponse::Response(ServiceResponse::new(
        res.into_parts().0,
        response.map_into_left_body(),
    )))
}

// Generic error handler.
fn get_error_response<B>(res: &ServiceResponse<B>, error: &str) -> HttpResponse {
    let request = res.request();

    // Provide a fallback to a simple plain text response in case an error occurs during the
    // rendering of the error page.
    let fallback = |e: &str| {
        HttpResponse::build(res.status())
            .content_type(ContentType::plaintext())
            .body(e.to_string())
    };

    let tera = request.app_data::<web::Data<Tera>>().map(|t| t.get_ref());
    match tera {
        Some(tera) => {
            let mut context = tera::Context::new();
            context.insert("error", error);
            context.insert("status_code", res.status().as_str());
            let body = tera.render("error.html", &context);

            match body {
                Ok(body) => HttpResponse::build(res.status())
                    .content_type(ContentType::html())
                    .body(body),
                Err(_) => fallback(error),
            }
        }
        None => fallback(error),
    }
}
