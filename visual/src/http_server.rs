//! http_server.rs


use std::fs;
use std::io::BufReader;
use std::str::FromStr;
use actix_files::NamedFile;
use actix_web::{web, App, HttpServer, Responder};
use common_lib::operator::{Msg};
use crossbeam_channel::Sender;
use handlebars::Handlebars;
use tokio::runtime::Handle;
use common_lib::init::ConfigLocation;
use crate::analysis;
// use crate::api_internals::request_index_data;




// /// GET '/'
// async fn index(tx: web::Data<Sender<Msg>>) -> impl Responder {
//     request_index_data(tx).await
// }


/// start actix in a new blocking thread
pub fn run(tx_operator2: Sender<Msg>, tokio_runtime: Handle) {
    std::thread::spawn(move ||{
        let _ = tokio_runtime.block_on(async {

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

            HttpServer::new(move || {
                App::new()
                    .app_data(tx_operator.clone())
                    .app_data(handlebars_ref.clone())
                    // .route("/", web::get().to(index))
                    // .route("/analysis", web::get().to(analysis::get_analysis))
                    .route("/", web::get().to(analysis::get_analysis))
                    .route("/js/chart.js", web::get().to(get_chart_js))

            })
            // .bind_rustls(("127.0.0.1", 8443), config)?
            .bind(("127.0.0.1", 8080))?
            .run()
            .await
        });
    });
}

/// GET http://127.0.0.1:8080/js/chart.js
/// https://www.chartjs.org/docs/latest/getting-started/installation.html
async fn get_chart_js()-> impl Responder{
    tracing::debug!("[get_chart_js]");
    let config_location:ConfigLocation = ConfigLocation::from_str(&std::env::var("CONFIG_LOCATION").unwrap_or_else(|_| "not_docker".to_owned())).expect("CONFIG_LOCATION");

    match config_location{
        ConfigLocation::Docker => NamedFile::open_async("./static/js/chart.js").await,
        // TODO: un-hard-code paths
        ConfigLocation::NotDocker => NamedFile::open_async("visual/static/js/chart.js").await
    }

}

fn load_private_key(filename: &str) -> rustls::PrivateKey {
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

    panic!(
        "no keys found in {:?} (encrypted keys not supported)",
        filename
    );
}

fn load_certs(filename: &str) -> Vec<rustls::Certificate> {
    let certfile = fs::File::open(filename).expect("cannot open certificate file");
    let mut reader = BufReader::new(certfile);
    rustls_pemfile::certs(&mut reader)
        .unwrap()
        .iter()
        .map(|v| rustls::Certificate(v.clone()))
        .collect()
}

// fn load_ocsp(filename: &Option<String>) -> Vec<u8> {
//     let mut ret = Vec::new();
//
//     if let Some(name) = filename {
//         fs::File::open(name)
//             .expect("cannot open ocsp file")
//             .read_to_end(&mut ret)
//             .unwrap();
//     }
//
//     ret
// }


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

