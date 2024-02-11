#[macro_use]
extern crate rocket;

mod error;
mod hr;
mod messages;

#[route(GET, uri = "/")]
async fn index() -> &'static str {
    "TheBeat"
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    use rocket::http::Method;
    use rocket_cors::{AllowedOrigins, CorsOptions};
    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(
            vec![Method::Get, Method::Post, Method::Patch, Method::Delete]
                .into_iter()
                .map(From::from)
                .collect(),
        )
        .allow_credentials(true);

    let _rocket = rocket::build()
        .attach(cors.to_cors().unwrap())
        .manage(hr::Database::new())
        .mount("/api/v1", hr::get_routes())
        .ignite()
        .await?
        .launch()
        .await?;

    Ok(())
}
