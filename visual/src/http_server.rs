//! http_server.rs


use std::fs;
use std::io::{BufReader, Read};
use std::str::FromStr;
use actix_files::NamedFile;
use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use crossbeam_channel::Sender;
use handlebars::Handlebars;
use serde_json::json;

use tokio::try_join;
use common_lib::init::ConfigLocation;
use common_lib::Msg;
use crate::analysis::{present_chart_multi_line};
use crate::api_internals::request_raw_data;
use crate::chat_main::{get_chat, chat_ws};
use crate::chat_server::ChatServer;


/// GET '/raw'
async fn get_raw(tx: web::Data<Sender<Msg>>) -> impl Responder {
    request_raw_data(tx).await
}



/// start actix in a new blocking thread
pub async fn run(tx_operator2: Sender<Msg>) -> Result<(), std::io::Error> {

    // handlebars
    // refs:
    // https://github.com/actix/examples/blob/master/templating/handlebars/src/main.rs
    // https://github.com/sunng87/handlebars-rust/tree/master/examples
    let mut handlebars = Handlebars::new();
    let config_location:ConfigLocation = ConfigLocation::from_str(&std::env::var("CONFIG_LOCATION").unwrap_or_else(|_| "not_docker".to_owned())).expect("CONFIG_LOCATION");
    // or CARGO_PKG_NAME
    let package_name = env!("CARGO_MANIFEST_DIR");
    let handlebar_static_path = match config_location {
        ConfigLocation::Docker => "./static/templates".to_string(),
        ConfigLocation::NotDocker => {
            // frontend/static/templates
            // /Users/xyz/.../trade/frontend/static/templates
            format!("{}/static/templates", &package_name)
        }
    };
    tracing::debug!("[web_server] registering handlebars static files to: {}",&handlebar_static_path);
    handlebars.register_templates_directory(".html", handlebar_static_path).unwrap();
    let handlebars_ref = web::Data::new(handlebars);


    // TLS
    // https://github.com/rustls/rustls/blob/main/examples/src/bin/tlsserver-mio.rs
    // let certs = load_certs("/Users/gp/trade/frontend/certificate/cert.pem");
    // let privkey = load_private_key("/Users/gp/trade/frontend/certificate/key.pem");
    // let (certs, privkey) = match config_location{
    //     ConfigLocation::Docker => {
    //         let certs = load_certs("./static/certificate/cert.pem");
    //         let privkey = load_private_key("./static/certificate/key.pem");
    //         // let ocsp = load_ocsp(&args.flag_ocsp);
    //         (certs, privkey)
    //     },
    //     ConfigLocation::NotDocker => {
    //         // TODO: don't hard code paths
    //         let certs = load_certs("visual/static/certificate/cert.pem");
    //         let privkey = load_private_key("visual/static/certificate/key.pem");
    //         // let ocsp = load_ocsp(&args.flag_ocsp);
    //         (certs, privkey)
    //     }
    // };

    // let config = rustls::ServerConfig::builder()
    //     .with_cipher_suites(&rustls::ALL_CIPHER_SUITES.to_vec())
    //     .with_safe_default_kx_groups()
    //     .with_protocol_versions(&rustls::ALL_VERSIONS.to_vec())
    //     .expect("inconsistent cipher-suites/versions specified")
    //     // .with_client_cert_verifier(NoClientAuth::)
    //     // .with_single_cert_with_ocsp(certs, privkey, ocsp)
    //     .with_no_client_auth()
    //     .with_single_cert(certs, privkey)
    //     .expect("bad certificates/private key");

    let tx_operator = web::Data::new(tx_operator2.clone());

    let (chat_server, server_tx) = ChatServer::new();

    let chat_server = tokio::spawn(chat_server.run());

    let http_server = HttpServer::new(move || {
        App::new()
            .app_data(tx_operator.clone())
            .app_data(handlebars_ref.clone())
            .app_data(web::Data::new(server_tx.clone()))
            .route("/", web::get().to(present_chart_multi_line))
            .route("/js/chart.js", web::get().to(get_chart_js))
            .route("/js/chartjs-adapter-date-fns.js", web::get().to(get_chart_js_date))
            // .route("/ws", web::get().to(ws_index))
            .route("/raw", web::get().to(get_raw))
            .route("/chat", web::get().to(get_chat))
            .route("/chart_ws", web::get().to(get_chart_ws))
            .route("/ws", web::get().to(chat_ws))

    })
    // .bind_rustls(("127.0.0.1", 8443), config)?
    .bind(("127.0.0.1", 8080))?
    .run();

    try_join!(http_server, async move { chat_server.await.unwrap() })?;

    // http_server.await;

    Ok(())

}



// pub async fn get_chart_ws() -> impl Responder {
//     NamedFile::open_async("visual/static/templates/chart_ws.html").await.unwrap()
// }

/// show multiple datasets on the same chart, regardless of x-axis count
pub async fn get_chart_ws(_tx_db: web::Data<Sender<Msg>>, hb: web::Data<Handlebars<'_>>/*, session: Session*/) -> HttpResponse {
    // let tx_db = tx_db.into_inner().as_ref().clone();

    let data = json!({
        "title": "chart_ws",
        "parent": "base0",
        "is_logged_in": true,
        "chart_title": "TEST_WS",
        // "data_vec": data_vec_json,
    });
    let body = hb.render("chart_ws", &data).unwrap();
    HttpResponse::Ok().append_header(("cache-control", "no-store")).body(body)

}
















/// GET http://127.0.0.1:8080/js/chart.js
/// https://www.chartjs.org/docs/latest/getting-started/installation.html
async fn get_chart_js()-> impl Responder{
    // tracing::debug!("[get_chart_js]");
    let config_location:ConfigLocation = ConfigLocation::from_str(&std::env::var("CONFIG_LOCATION").unwrap_or_else(|_| "not_docker".to_owned())).expect("CONFIG_LOCATION");

    match config_location{
        ConfigLocation::Docker => NamedFile::open_async("./static/js/chart.js").await,
        // TODO: un-hard-code paths
        ConfigLocation::NotDocker => NamedFile::open_async("visual/static/js/chart.js").await
    }

}

/// GET http://127.0.0.1:8080/js/chart.js
/// https://www.chartjs.org/docs/latest/getting-started/installation.html
/// TODO: un-hard-code paths
async fn get_chart_js_date()-> impl Responder{
    // tracing::debug!("[get_chart_js]");
    let config_location:ConfigLocation = ConfigLocation::from_str(&std::env::var("CONFIG_LOCATION").unwrap_or_else(|_| "not_docker".to_owned())).expect("CONFIG_LOCATION");
    match config_location{
        ConfigLocation::Docker => NamedFile::open_async("./static/js/chartjs-adapter-date-fns.js").await,
        ConfigLocation::NotDocker => NamedFile::open_async("visual/static/js/chartjs-adapter-date-fns.js").await
    }

}

// /// GET http://127.0.0.1:8080/js/chart.js
// /// https://www.chartjs.org/docs/latest/getting-started/installation.html
// async fn get_stocks_csv()-> impl Responder{
//     // tracing::debug!("[get_chart_js]");
//     let config_location:ConfigLocation = ConfigLocation::from_str(&std::env::var("CONFIG_LOCATION").unwrap_or_else(|_| "not_docker".to_owned())).expect("CONFIG_LOCATION");
//
//     match config_location{
//         ConfigLocation::Docker => NamedFile::open_async("./static/js/chart.js").await,
//         // TODO: un-hard-code paths
//         ConfigLocation::NotDocker => NamedFile::open_async("visual/static/stocks.csv").await
//     }
// }


fn _load_private_key(filename: &str) -> rustls::PrivateKey {
    let keyfile = fs::File::open(filename).expect("cannot open private key file");
    let mut reader = BufReader::new(keyfile);

    loop {
        match rustls_pemfile::read_one(&mut reader).expect("cannot parse private key .pem file") {
            Some(rustls_pemfile::Item::RSAKey(key)) => return rustls::PrivateKey(key),
            Some(rustls_pemfile::Item::PKCS8Key(key)) => return rustls::PrivateKey(key),
            Some(rustls_pemfile::Item::ECKey(key)) => return rustls::PrivateKey(key),
            None => break,
            _ => {}
        }
    }

    panic!("no keys found in {:?} (encrypted keys not supported)", filename);
}

fn _load_certs(filename: &str) -> Vec<rustls::Certificate> {
    let certfile = fs::File::open(filename).expect("cannot open certificate file");
    let mut reader = BufReader::new(certfile);
    rustls_pemfile::certs(&mut reader)
        .unwrap()
        .iter()
        .map(|v| rustls::Certificate(v.clone()))
        .collect()
}

fn _load_ocsp(filename: &Option<String>) -> Vec<u8> {
    let mut ret = Vec::new();

    if let Some(name) = filename {
        fs::File::open(name)
            .expect("cannot open ocsp file")
            .read_to_end(&mut ret)
            .unwrap();
    }

    ret
}


#[cfg(test)]
mod tests {
    // use actix_web::{http::header::ContentType, test, App};

    // use super::*;

    // #[actix_web::test]
    // async fn test_index_get() {
    //     let app = test::init_service(App::new().service(index)).await;
    //     let req = test::TestRequest::default()
    //         .insert_header(ContentType::plaintext())
    //         .to_request();
    //     let resp = test::call_service(&app, req).await;
    //     assert!(resp.status().is_success());
    // }
    //
    // #[actix_web::test]
    // async fn test_index_post() {
    //     let app = test::init_service(App::new().service(index)).await;
    //     let req = test::TestRequest::post().uri("/").to_request();
    //     let resp = test::call_service(&app, req).await;
    //     assert!(resp.status().is_client_error());
    // }
}

