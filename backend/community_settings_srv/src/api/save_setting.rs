use actix_web::{post, web, Responder, http::header};

use crate::cli::Cli;
use crate::file_util;

const PAYLOAD_LIMIT: usize = 10_000_000; // 10 Megabyte

#[post("/api/setting")]
pub async fn save_setting_handler(
    data: web::Payload,
    content_type: web::Header<header::ContentType>,
    cli: web::Data<&'static Cli>,
) -> std::io::Result<impl Responder> {
    println!("Content-Type: {}", content_type.to_string());
    let bytes = match data.to_bytes_limited(PAYLOAD_LIMIT).await {
        Ok(Ok(x)) => x,
        Ok(Err(e)) => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("wut: {}", e))),
        Err(_e) => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "too many bytes in payload")),
    };
    let next_id = file_util::next_setting_id(&cli.folder);
    let parsed_data = if super::is_mime_type_ron_capable(&content_type) {
        // Parse as RON
        match ron::de::from_reader(bytes.as_ref()) {
            Ok(x) => x,
            Err(e) => {
                let e_msg = format!("{}", e);
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e_msg));
            }
        }
    } else {
        // Parse JSON (fallback)
        match serde_json::from_reader(bytes.as_ref()) {
            Ok(x) => x,
            Err(e) => {
                let e_msg = format!("{}", e);
                if let Some(io_e) = e.io_error_kind() {
                    return Err(std::io::Error::new(io_e, e_msg));
                } else {
                    return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e_msg));
                }

            }
        }
    };
    // TODO validate user and app id
    // Reject blocked users and apps
    let path = file_util::setting_path_by_id(&cli.folder, next_id, file_util::RON_EXTENSION);
    let writer = std::io::BufWriter::new(std::fs::File::create(path)?);
    if let Err(e) = ron::ser::to_writer(writer, &parsed_data) {
        let e_msg = format!("{}", e);
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e_msg));
    }

    let path = file_util::setting_path_by_id(&cli.folder, next_id, file_util::JSON_EXTENSION);
    let writer = std::io::BufWriter::new(std::fs::File::create(path)?);
    if let Err(e) = serde_json::to_writer(writer, &parsed_data) {
        let e_msg = format!("{}", e);
        if let Some(io_e) = e.io_error_kind() {
            return Err(std::io::Error::new(io_e, e_msg));
        } else {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e_msg));
        }
    }

    // TODO create symlinks for other ways of looking up these settings files

    Ok(actix_web::HttpResponse::NoContent())
}
