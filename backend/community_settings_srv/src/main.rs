mod api;
mod cli;
mod file_util;

use actix_web::{web, App, HttpServer};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = cli::Cli::get();
    println!("cli: {:?}", args);

    simplelog::WriteLogger::init(
        #[cfg(debug_assertions)]
        {
            log::LevelFilter::Debug
        },
        #[cfg(not(debug_assertions))]
        {
            log::LevelFilter::Info
        },
        Default::default(),
        std::fs::File::create(&args.log).expect("Failed to create log file"),
        //std::fs::File::create("/home/deck/powertools-rs.log").unwrap(),
    )
    .unwrap();
    log::debug!("Logging to: {}", args.log.display());

    let leaked_args: &'static cli::Cli = Box::leak::<'static>(Box::new(args));
    HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(leaked_args))
                //.app_data(web::Data::new(IndexPage::load("dist/index.html").unwrap()))
                //.app_data(basic::Config::default().realm("Restricted area"))
                .service(api::get_setting_by_id)
                .service(api::save_setting_with_new_id)
        })
            .bind(("0.0.0.0", leaked_args.port))?
            .run()
            .await
}
