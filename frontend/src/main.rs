use actix_files::Files;
use actix_web::{
    body::BoxBody,
    dev::ServiceResponse,
    error,
    http::{header::ContentType, StatusCode},
    middleware::{self, ErrorHandlerResponse, ErrorHandlers},
    web, App, Error, HttpResponse, HttpServer, Responder, Result,
};
use actix_web_lab::respond::Html;
use clap::Parser;
use log::info;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::io;
use std::path::Path;
use std::process::Command;
use std::str::FromStr;
use std::string::ToString;
use std::sync::Mutex;
use strum_macros::Display;
use strum_macros::EnumString;
use tera::Tera;

#[derive(Copy, Clone, Debug, Deserialize, EnumString, Display, PartialEq, Hash, Eq)]
enum Location {
    IT,
    DE,
}

#[derive(Debug, Deserialize)]
struct ConnectQuery {
    location: Location,
}

// store tera template in application state
async fn index(tmpl: web::Data<tera::Tera>) -> Result<impl Responder, Error> {
    let s = tmpl
        .render("index.html", &tera::Context::new())
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;

    Ok(Html(s))
}

// TODO: use an eum for the location
async fn is_connected_to_location(location: &Location) -> Result<bool, Error> {
    let client = awc::Client::default();
    let req = client.get("http://ifconfig.io/all.json");
    let mut res = req
        .send()
        .await
        .map_err(|e| error::ErrorInternalServerError(e.to_string()))?;
    let body = res
        .json::<serde_json::Value>()
        .await
        .map_err(|e| error::ErrorInternalServerError(e.to_string()))?;
    let country_code = body
        .get("country_code")
        .ok_or_else(|| error::ErrorInternalServerError("country_code not found"))?;
    let location_from_country_code = Location::from_str(country_code.as_str().unwrap())
        .map_err(|e| error::ErrorInternalServerError(e.to_string()))?;
    if location_from_country_code == *location {
        Ok(true)
    } else {
        Ok(false)
    }
}

async fn is_connected(
    query: web::Query<ConnectQuery>,
    data: web::Data<AppState>,
) -> Result<impl Responder, Error> {
    let l = &query.location;

    if is_connected_to_location(l).await? {
        let mut running_processes = data.running_processes.lock().unwrap();
        running_processes.remove(&l);
        Ok(format!("Already connected to {}.", l))
    } else {
        if data.running_processes.lock().unwrap().get(&l).is_some() {
            Ok(format!("Connection to {} in progress.", l))
        } else {
            Ok(format!("Connection to {} not started.", l))
        }
    }
}

fn create_vpn_server(path: &str, region: &Location) -> Result<u32, io::Error> {
    let dir = Path::new(path).parent().unwrap();
    env::set_current_dir(&dir);
    let file_name = Path::new(path);
    Command::new(file_name)
        .arg(region.to_string().as_str())
        .spawn()
        .map(|x| x.id())
}

async fn connect(
    query: web::Query<ConnectQuery>,
    data: web::Data<AppState>,
) -> Result<impl Responder, Error> {
    match query.location {
        l if l == Location::IT || l == Location::DE => {
            if is_connected_to_location(&l).await? {
                let mut running_processes = data.running_processes.lock().unwrap();
                running_processes.remove(&l);
                Ok(format!("Already connected to {}.", &l))
            } else {
                if data.running_processes.lock().unwrap().get(&l).is_some() {
                    Ok(format!("Connection to {} in progress.", &l))
                } else {
                    let id = create_vpn_server(&data.vpn_setup_path, &l)
                        .map_err(|e| error::ErrorInternalServerError(e.to_string()))?;

                    let mut running_processes = data.running_processes.lock().unwrap();
                    running_processes.insert(l, id);
                    Ok(format!("Started connection."))
                }
            }
        }
        l => Err(error::ErrorBadRequest(format!(
            "Connection to {} not supported.",
            &l
        ))),
    }
}

struct AppState {
    vpn_setup_path: String,
    running_processes: Mutex<HashMap<Location, u32>>, // <- Mutex is necessary to mutate safely across threads
}

#[derive(Parser, Debug)]
#[command(author, version)]
struct Args {
    /// Name of the person to greet
    #[arg(long)]
    vpn_setup_path: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let args = Args::parse();

    let port = 8080;
    info!("starting HTTP server at http://localhost:{}", port);

    let app_state = web::Data::new(AppState {
        vpn_setup_path: args.vpn_setup_path,
        running_processes: Mutex::new(HashMap::new()),
    });

    HttpServer::new(move || {
        let tera = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();

        App::new()
            .app_data(app_state.clone())
            .app_data(web::Data::new(tera))
            .wrap(middleware::Logger::default())
            .service(Files::new("/img", "static/img/").show_files_listing())
            .service(Files::new("/src", "static/src/").show_files_listing())
            .service(web::resource("/connect").route(web::get().to(connect)))
            .service(web::resource("/is_connected").route(web::get().to(is_connected)))
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
