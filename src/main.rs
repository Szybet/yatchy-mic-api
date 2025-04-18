use actix_web::{App, HttpServer, web::Data};
use api::AppState;
use log::*;
use speech::get_text_from_wav;
use text_match::text_match;
use tokio::{
    process::Command,
    sync::{Mutex, mpsc},
};
use wav::boost_wav;

mod api;
mod speech;
mod text_match;
mod wav;

use std::sync::{Arc, Once};
static INIT: Once = Once::new();

fn init_log() {
    INIT.call_once(|| {
        env_logger::init_from_env(
            env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "debug"),
        );
    });
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_log();
    info!("Start");

    let (main_tx, mut main_rx) = mpsc::channel(32);
    let (server_tx, mut server_rx) = mpsc::channel(32);

    // Create shared state
    let app_state = Arc::new(AppState {
        main_tx,
        server_rx: Mutex::new(server_rx),
    });

    let server_task = tokio::spawn(async move {
        HttpServer::new(move || {
            App::new()
                .app_data(Data::new(app_state.clone()))
                .service(api::root)
                .service(api::start_mic)
                .service(api::stop_mic)
        })
        .bind("0.0.0.0:14679")
        .unwrap()
        .run()
        .await
        .unwrap();
    });

    let mut child_process: Option<tokio::process::Child> = None;

    // Signal processing loop
    while let Some(signal) = main_rx.recv().await {
        match signal {
            api::MainSignal::StartMic => {
                debug!("Main received startMic");
                if child_process.is_none() {
                    match Command::new("data/mic.sh").spawn() {
                        Ok(child) => {
                            child_process = Some(child);
                            debug!("Process started successfully");
                        }
                        Err(e) => {
                            error!("Failed to start process: {}", e);
                        }
                    }
                } else {
                    debug!("Process is already running");
                }
            }
            api::MainSignal::StopMic => {
                debug!("Main received stopMic");
                if let Some(mut child) = child_process.take() {
                    if let Err(e) = child.kill().await {
                        error!("Failed to kill process: {}", e);
                    }
                    // Wait for process to exit
                    let _ = child.wait().await;
                    debug!("Process ended");

                    Command::new("data/repair.sh")
                        .spawn()
                        .unwrap()
                        .wait()
                        .await
                        .unwrap();
                    debug!("Repaired wav");

                    let str_from_wav = get_text_from_wav("/tmp/wavs/repaired.wav".to_string());
                    debug!("Received words: {}", str_from_wav);

                    // Events
                    let spec_words = vec!["light", "party", "call"];
                    let mut spec_comply: Vec<f32> = Vec::new();
                    for word in spec_words.clone() {
                        spec_comply.push(text_match(str_from_wav.clone(), word.to_string()));
                    }

                    let mut max = 0.0;
                    let mut j = 0;
                    for (iter, spec) in spec_comply.iter().enumerate() {
                        if *spec > max {
                            max = *spec;
                            j = iter;
                        }
                    }

                    debug!("Words: {:?}, match: {:?}", spec_words, spec_comply);
                    if max < 0.7 {
                        error!("Max value is very low!");
                    }
                    server_tx
                        .send(api::ServerSignal::MainAnswer(spec_words[j].to_string()))
                        .await
                        .unwrap();
                } else {
                    error!("Failed to obtain child");
                    server_tx
                        .send(api::ServerSignal::MainAnswer("ERR".to_string()))
                        .await
                        .unwrap();
                }
            }
        }
    }

    info!("Server stopped");
    Ok(())
}

#[test]
fn test_speech() {
    init_log();

    let str = get_text_from_wav("test_stuff/jfk.wav".to_string());
    debug!("Str received: {}", str);
}

#[test]
fn test_text_match() {
    init_log();

    let str1 = String::from("Light");
    let str2 = String::from("I don't know what light light is the meaning of words");

    text_match(str1, str2);
}

#[test]
fn test_boost_wav() {
    init_log();

    boost_wav("/tmp/wavs/repaired.wav", 5.0).unwrap();
}

#[test]
fn test_speech_tmp() {
    init_log();

    let str = get_text_from_wav("/tmp/wavs/repaired.wav".to_string());
    debug!("Str received: {}", str);
}
