//
// This file is part of Almond
//
// Copyright 2021 The Board of Trustees of the Leland Stanford Junior University
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// Author: Giovanni Campagna <gcampagn@cs.stanford.edu>

use std::result::Result;
use futures;
use futures::stream::{StreamExt, TryStreamExt, SplitSink};
use futures::sink::SinkExt;
use rustyline::{error::ReadlineError, Editor};
use tokio::task;
use serde_json::{Value, json};
use tokio_tungstenite::WebSocketStream;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite;
use std::sync::{Arc, Mutex};

mod error;
use crate::error::AlmondError;

async fn handle_server_message(msg : Value) {
    println!("Received {}", msg);

    let type_ = msg["type"].as_str().unwrap();
    match type_ {
        "id" => {
            // TODO do something
            return;
        }

        "askSpecial" => {
            // TODO do something
            return;
        }

        "text" => {
            println!("Almond says: {}", msg["text"].as_str().unwrap());
        }

        "command" => {
            println!(">> {}", msg["command"].as_str().unwrap());
        }

        // TODO handle the other messages
        &_ => {
            println!(">> unhandled server message of type {}", type_);
        }
    };
}

async fn handle_thingtalk(ws : &mut SplitSink<WebSocketStream<TcpStream>, tungstenite::Message>, line : &str) -> Result<(), AlmondError> {
    ws.send(tungstenite::Message::Text(json!({
        "type": "tt",
        "code": line
    }).to_string())).await?;
    Ok(())
}

async fn handle_command(ws : &mut SplitSink<WebSocketStream<TcpStream>, tungstenite::Message>, line : &str) -> Result<(), AlmondError> {
    ws.send(tungstenite::Message::Text(json!({
        "type": "command",
        "text": line
    }).to_string())).await?;
    Ok(())
}

async fn program() -> Result<(), AlmondError> {
    let (client, _) = tokio_tungstenite::connect_async("ws://127.0.0.1:3000/api/conversation").await?;
    let (mut sender, mut receiver) = client.split();

    let reader = async move {
        while let Some(msg) = receiver.try_next().await? {
            if let tungstenite::Message::Text(json) = msg {
                handle_server_message(serde_json::from_str(&json).unwrap()).await;
            }
        }

        Ok::<(), AlmondError>(())
    };

    let writer = async move {
        let rl = Arc::new(Mutex::new(Editor::<()>::new()));
        loop {
            let rl2 = rl.clone();
            let readline = task::spawn_blocking(move || {
                rl2.lock().unwrap().readline(">> ")
            }).await?;
            match readline {
                Ok(line) => {
                    let trimmed = line.trim();
                    if trimmed.len() == 0 {
                        continue;
                    }

                    if trimmed.starts_with("\t ") {
                        handle_thingtalk(&mut sender, &trimmed[3..]).await?;
                    } else {
                        handle_command(&mut sender, &trimmed).await?;
                    }
                }

                Err(ReadlineError::Interrupted) => {
                    sender.close().await?;
                    return Ok::<(), AlmondError>(());
                }

                Err(ReadlineError::Eof) => {
                    sender.close().await?;
                    return Ok::<(), AlmondError>(());
                }

                Err(err) => {
                    return Err::<(), AlmondError>(From::from(err));
                }
            }
        }
    };

    futures::try_join!(reader, writer)?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), AlmondError> {
    program().await?;

    Ok(())
}
