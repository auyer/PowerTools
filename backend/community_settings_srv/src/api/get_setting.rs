use actix_web::{get, web, Responder, http::header};

use crate::cli::Cli;
use crate::file_util;

fn special_settings() -> community_settings_core::v1::Metadata {
    community_settings_core::v1::Metadata {
        name: "Zeroth the Least".to_owned(),
        steam_app_id: 1675200,
        steam_user_id: 76561198116690523,
        steam_username: "NGnius".to_owned(),
        tags: vec!["0".to_owned(), "gr8".to_owned()],
        id: 0.to_string(),
        config: community_settings_core::v1::Config {
            cpus: vec![
                community_settings_core::v1::Cpu {
                    online: true,
                    clock_limits: Some(community_settings_core::v1::MinMax { max: Some(1), min: Some(0) }),
                    governor: "Michaëlle Jean".to_owned(),
                },
                community_settings_core::v1::Cpu {
                    online: false,
                    clock_limits: Some(community_settings_core::v1::MinMax { max: Some(1), min: Some(0) }),
                    governor: "Adrienne Clarkson".to_owned(),
                },
                community_settings_core::v1::Cpu {
                    online: true,
                    clock_limits: Some(community_settings_core::v1::MinMax { max: Some(1), min: Some(0) }),
                    governor: "Michael Collins".to_owned(),
                }
            ],
            gpu: community_settings_core::v1::Gpu {
                fast_ppt: Some(1),
                slow_ppt: Some(1),
                tdp: None,
                tdp_boost: None,
                clock_limits: Some(community_settings_core::v1::MinMax { max: Some(1), min: Some(0) }),
                slow_memory: false,
            },
            battery: community_settings_core::v1::Battery {
                charge_rate: Some(42),
                charge_mode: Some("nuclear fusion".to_owned()),
                events: vec![
                    community_settings_core::v1::BatteryEvent {
                        trigger: "anything but one on a gun".to_owned(),
                        charge_rate: Some(42),
                        charge_mode: Some("neutral".to_owned()),
                    }
                ],
            }
        }
    }
}

#[get("/api/setting/by_id/{id}")]
pub async fn get_setting_handler(
    id: web::Path<String>,
    accept: web::Header<header::Accept>,
    cli: web::Data<&'static Cli>,
) -> std::io::Result<impl Responder> {
    println!("Accept: {}", accept.to_string());
    let id: u128 = match id.parse() {
        Ok(x) => x,
        Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("invalid setting id `{}` (should be u128): {}", id, e))),
    };
    let preferred = accept.preference();
    if super::is_mime_type_ron_capable(&preferred) {
        // Send RON
        let ron = if id != 0 {
            let path = file_util::setting_path_by_id(&cli.folder, id, file_util::RON_EXTENSION);
            if !path.exists() {
                return Err(std::io::Error::new(std::io::ErrorKind::NotFound, format!("setting id {} does not exist", id)));
            }
            // TODO? cache this instead of always loading it from file
            let reader = std::io::BufReader::new(std::fs::File::open(path)?);
            match ron::de::from_reader(reader) {
                Ok(x) => x,
                Err(e) => {
                    let e_msg = format!("{}", e);
                    return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e_msg));
                }
            }
        } else {
            special_settings()
        };
        // TODO don't dump to string
        let result_body = ron::ser::to_string(&ron).unwrap();
        Ok(actix_web::HttpResponse::Ok()
            //.insert_header(header::ContentType("application/ron".parse().unwrap()))
            .insert_header(header::ContentType(mime::STAR_STAR))
            .body(actix_web::body::BoxBody::new(result_body))
        )
    } else {
        // Send JSON (fallback)
        let json = if id != 0 {
            let path = file_util::setting_path_by_id(&cli.folder, id, file_util::JSON_EXTENSION);
            // TODO? cache this instead of always loading it from file
            let reader = std::io::BufReader::new(std::fs::File::open(path)?);
            match serde_json::from_reader(reader) {
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
        } else {
            special_settings()
        };
        // TODO don't dump to string
        let result_body = serde_json::to_string(&json).unwrap();
        Ok(actix_web::HttpResponse::Ok()
            .insert_header(header::ContentType::json())
            .body(actix_web::body::BoxBody::new(result_body))
        )
    }
}
