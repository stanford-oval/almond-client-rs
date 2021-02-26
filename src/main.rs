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

//use tokio::io;

//use std::env;
use std::error::Error;
use std::result::Result;
//use std::net::SocketAddr;
use futures;
use futures::stream::TryStreamExt;
use futures::sink::SinkExt;
use rustyline::Editor;
use tokio::task;
use serde_json::{Value, json};
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message;
use tokio::net::TcpStream;

async fn handle_server_message(msg : Value) {
    println!("Received {}", msg);

    match msg["type"].as_str().unwrap() {
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
    };
}

async fn handle_thingtalk(ws : &mut WebSocketStream<TcpStream>, line : &str) {
    ws.send(Message::Text(json!({
        "type": "tt",
        "code": line
    }).to_string())).await;
}

async fn handle_command(ws : &mut WebSocketStream<TcpStream>, line : &str) {
    ws.send(Message::Text(json!({
        "type": "command",
        "text": line
    }).to_string())).await;
}

async fn program() -> Result<(), Box<dyn Error>> {
    let (mut client, _) = tokio_tungstenite::connect_async("ws://127.0.0.1:3000/api/conversation").await?;

    let reader = task::spawn(async {
        while let read = client.try_next().await? {
            match read {
                Some(msg) => {
                    if let Message::Text(json) = msg {
                        handle_server_message(serde_json::from_str(&json).unwrap());
                    }
                }

                None => {
                    break
                }
            }
        }

        Ok(())
    });

    let writer = task::spawn_blocking(|| {
        let mut rl = Editor::<()>::new();
        while let line = rl.readline("$ ").unwrap() {
            if line.starts_with("\t ") {
                handle_thingtalk(&mut client, &line[3..]);
            } else {
                handle_command(&mut client, &line);
            }
        };
    });

    reader.await?;
    writer.await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    program().await?;

    Ok(())
}
