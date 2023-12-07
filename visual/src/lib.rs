//! lib.rs

pub mod http_server {

    use common_lib::operator::{Msg, VisualResultSet};
    use common_lib::cb_ticker::ProductId;
    use actix_web::{get, web, App, HttpServer, Responder};
    use crossbeam_channel::Sender;


    #[get("/")]
    async fn index(tx: web::Data<Sender<Msg>>) -> impl Responder {

        let (tx_web, rx_web) = tokio::sync::oneshot::channel::<VisualResultSet>();

        // match tx.send(Msg::GetChartForOne{key: ProductId::BtcUsd, sender:tx_web}) {
        match tx.send(Msg::GetChartForAll{key: ProductId::BtcUsd, sender:tx_web}) {
            Ok(_)=>{
                match rx_web.await {
                    Ok(visual_result_set)=>{

                        use datafusion::arrow::util::pretty::pretty_format_batches;

                        if let Some(df) = visual_result_set.data{
                            format!("{}", pretty_format_batches(&df.collect().await.unwrap()).unwrap().to_string())
                        } else {
                            format!("[index] no dataframe returned")
                        }

                    },
                    Err(e)=> {
                        tracing::error ! ("[index] receive error: {:?}", & e);
                        format!("[index] send error: {:?}", &e)
                    }
                }
            },
            Err(e)=> {
                tracing::error!("[index] send error: {:?}", &e);
                format!("[index] send error: {:?}", &e)
            }
        }
    }

    #[get("/{name}")]
    async fn hello(name: web::Path<String>) -> impl Responder {
        format!("Hello {}!", &name)
    }



    pub fn run(tx_operator2: Sender<Msg>)  {

        let tokio_runtime = tokio::runtime::Builder::new_multi_thread()
            // .worker_threads(7)
            .on_thread_start(|| {})
            .on_thread_stop(|| {})
            .thread_name("actix")
            .enable_all()
            .build()
            .expect("Tokio runtime didn't start");

        let _ = tokio_runtime.block_on(async {

            let tx_operator = web::Data::new(tx_operator2.clone());

            HttpServer::new(move ||
                App::new()
                    .app_data(tx_operator.clone())
                    .service(index)
                    .service(hello))
                .bind(("127.0.0.1", 8080))?
                .run()
                .await

        });

    }


}

#[cfg(test)]
mod tests {

    use actix_web::{http::header::ContentType, test, App};

    use super::*;

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