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
use serde_json::{Value, json};
use tokio_tungstenite::WebSocketStream;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite;
use tokio::sync::Mutex;
use std::sync::Arc;

use crate::error::AlmondError;

pub trait MessageHandler {
    fn on_new_text_message(self : &mut Self, msg : &str) -> ();
    fn on_new_command(self : &mut Self, msg : &str) -> ();
    fn set_expected(self : &mut Self, expected : Option<&str>) -> ();
}

pub struct AlmondClient {
    handler : Box<dyn MessageHandler + Send>,
    writer : SplitSink<WebSocketStream<TcpStream>, tungstenite::Message>,
    conversation_id : Option<String>,
    last_message_id : i64,
}

impl AlmondClient {
    pub async fn new(url : &str, handler : Box<dyn MessageHandler + Send>) -> Result<Arc<Mutex<AlmondClient>>, AlmondError> {
        let (client, _) = tokio_tungstenite::connect_async(url).await?;
        let (writer, mut reader) = client.split();
        let self_ = Arc::new(Mutex::new(AlmondClient { handler, writer, conversation_id: None, last_message_id: -1 }));

        let reader_self = self_.clone();
        tokio::spawn(async move {
            while let Some(msg) = reader.try_next().await? {
                if let tungstenite::Message::Text(json) = msg {
                    reader_self.lock().await.handle_server_message(serde_json::from_str(&json).unwrap()).await?;
                }
            }

            Ok::<(), AlmondError>(())
        });

        Ok(self_)
    }

    pub async fn send_thingtalk(self : &mut AlmondClient, line : &str) -> Result<(), AlmondError> {
        self.writer.send(tungstenite::Message::Text(json!({
            "type": "tt",
            "code": line
        }).to_string())).await?;
        Ok(())
    }

    pub async fn send_command(self : &mut AlmondClient, line : &str) -> Result<(), AlmondError> {
        self.writer.send(tungstenite::Message::Text(json!({
            "type": "command",
            "text": line
        }).to_string())).await?;
        Ok(())
    }

    pub async fn close(self : &mut AlmondClient) -> Result<(), AlmondError> {
        self.writer.close().await?;
        Ok(())
    }

    async fn handle_server_message(self : &mut AlmondClient, msg : Value) -> Result<(), AlmondError> {
        println!("Received {}", msg);

        let type_ = msg["type"].as_str().unwrap();

        if type_ == "id" {
            self.conversation_id = Some(msg["id"].as_str().ok_or(AlmondError::ProtocolError("missing conversation id"))?.to_string());
        } else if type_ == "askSpecial" {
            match &msg["what"] {
                Value::Null => {
                    self.handler.set_expected(None)
                }

                _ => {
                    self.handler.set_expected(Some(msg["what"].as_str().ok_or(AlmondError::ProtocolError("invalid ask_special what field"))?))
                }
            }
        } else {
            let id = msg["id"].as_i64().ok_or(AlmondError::ProtocolError("invalid message id"))?;
            if id <= self.last_message_id {
                return Ok(())
            }
            self.last_message_id = id;

            match type_ {
                "text" => {
                    self.handler.on_new_text_message(msg["text"].as_str().ok_or(AlmondError::ProtocolError("invalid text message"))?);
                }

                "command" => {
                    self.handler.on_new_command(msg["command"].as_str().ok_or(AlmondError::ProtocolError("invalid command message"))?);
                }

                &_ => {
                    // ignore other types of messages for now
                }
            }
        };

        Ok(())
    }
}
