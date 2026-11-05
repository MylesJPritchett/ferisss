#[macro_use]
extern crate rocket;

use env_logger::Env;
use log::info;
use rocket::fairing::AdHoc;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{Build, Rocket};
use serde_json::Value;
use std::env;

mod api;
mod logic;

use api::GameState;
use logic::MoveResponse;

#[get("/")]
fn handle_index() -> Json<Value> {
    Json(logic::info())
}

#[post("/start", format = "json", data = "<start_req>")]
fn handle_start(start_req: Json<GameState>) -> Status {
    logic::start(&start_req);
    Status::Ok
}

#[post("/move", format = "json", data = "<move_req>")]
fn handle_move(move_req: Json<GameState>) -> Json<MoveResponse> {
    Json(logic::get_move(&move_req))
}

#[post("/end", format = "json", data = "<end_req>")]
fn handle_end(end_req: Json<GameState>) -> Status {
    logic::end(&end_req);
    Status::Ok
}

#[launch]
fn rocket() -> _ {
    // Log at info unless `RUST_LOG` says otherwise.
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    info!("Starting Battlesnake Server...");

    server(env::var("PORT").ok().as_deref())
}

/// Builds the server, taking its port from `port` (the `PORT` variable
/// hosting platforms set) when given, otherwise from Rocket's own config.
fn server(port: Option<&str>) -> Rocket<Build> {
    let mut figment = rocket::Config::figment();
    if let Some(port) = port {
        let port: u16 = port
            .parse()
            .unwrap_or_else(|_| panic!("PORT must be a port number, got {port:?}"));
        figment = figment.merge(("port", port));
    }

    rocket::custom(figment)
        .attach(AdHoc::on_response("Server ID Middleware", |_, res| {
            Box::pin(async move {
                res.set_raw_header("Server", "battlesnake/github/ferisss");
            })
        }))
        .mount(
            "/",
            routes![handle_index, handle_start, handle_move, handle_end],
        )
}

#[cfg(test)]
mod tests {
    use super::server;
    use rocket::local::blocking::Client;

    #[test]
    fn responses_identify_ferisss_in_the_server_header() {
        let client = Client::tracked(server(None)).unwrap();
        let response = client.get("/").dispatch();
        assert_eq!(
            response.headers().get_one("Server"),
            Some("battlesnake/github/ferisss")
        );
    }

    #[test]
    fn port_defaults_to_8000() {
        let port: u16 = server(None).figment().extract_inner("port").unwrap();
        assert_eq!(port, 8000);
    }

    #[test]
    fn port_variable_overrides_the_default() {
        let port: u16 = server(Some("9000"))
            .figment()
            .extract_inner("port")
            .unwrap();
        assert_eq!(port, 9000);
    }
}
