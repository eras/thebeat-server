#[macro_use]
extern crate rocket;
use rocket::fs::FileServer;

use rocket::config::Config;
use rocket::log::LogLevel;
use rocket_slogger::{o, Drain, Logger, Slogger};
use slog_term::{FullFormat, PlainSyncDecorator};
use std::io;

mod error;
mod expiring;
mod hr;
mod messages;

fn convert_time_fmt_error(cause: time::error::Format) -> io::Error {
    io::Error::new(io::ErrorKind::Other, cause)
}
const TIMESTAMP_FORMAT: &[time::format_description::FormatItem] = time::macros::format_description!(
    "[year repr:full]-[month repr:numerical padding:zero]-[day padding:zero] [hour repr:24 padding:zero]:[minute padding:zero]:[second padding:zero].[subsecond digits:3]"
);

fn timestamp_format(io: &mut dyn io::Write) -> io::Result<()> {
    let now: time::OffsetDateTime = std::time::SystemTime::now().into();
    write!(
        io,
        "{}",
        now.format(TIMESTAMP_FORMAT)
            .map_err(convert_time_fmt_error)?
    )
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    use rocket::http::Method;
    use rocket_cors::{AllowedOrigins, CorsOptions};

    let plain = PlainSyncDecorator::new(std::io::stdout());
    let logger = Logger::root(
        FullFormat::new(plain)
            .use_custom_timestamp(timestamp_format)
            .build()
            .fuse(),
        o!(),
    );
    let slogger_fairing = Slogger::from_logger(logger);

    let mut config = Config::from(Config::figment());
    config.log_level = LogLevel::Off;

    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(
            vec![Method::Get, Method::Post, Method::Patch, Method::Delete]
                .into_iter()
                .map(From::from)
                .collect(),
        )
        .allow_credentials(true);

    let _rocket = rocket::custom(config)
        .attach(slogger_fairing)
        .attach(cors.to_cors().unwrap())
        .manage(hr::Database::new())
        .mount("/", FileServer::from("static"))
        .mount("/api/v1", hr::get_routes())
        .ignite()
        .await?
        .launch()
        .await?;

    Ok(())
}
