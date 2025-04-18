use std::sync::{Arc};

use actix_web::{dev::Server, get, web, App, HttpResponse, HttpServer, Responder};
use tokio::sync::{mpsc, Mutex};

pub enum MainSignal {
    StartMic,
    StopMic,
}

pub enum ServerSignal {
    MainAnswer(String),
}

pub struct AppState {
    pub main_tx: mpsc::Sender<MainSignal>,
    pub server_rx: Mutex<mpsc::Receiver<ServerSignal>>,
}

#[get("/")]
async fn root(data: web::Data<Arc<AppState>>) -> impl Responder {
    HttpResponse::Ok().body("OK")
}

#[get("/start_mic")]
async fn start_mic(data: web::Data<Arc<AppState>>) -> impl Responder {
    data.main_tx.send(MainSignal::StartMic).await.unwrap();
    HttpResponse::Ok().body("OK")
}

#[get("/stop_mic")]
async fn stop_mic(data: web::Data<Arc<AppState>>) -> impl Responder {
    data.main_tx.send(MainSignal::StopMic).await.unwrap();
    let server_signal = data.server_rx.lock().await.recv().await.unwrap();
    match server_signal {
        ServerSignal::MainAnswer(x) => {
            return HttpResponse::Ok().body(x)
        },
    }
    // HttpResponse::Ok().body("ERR")
}