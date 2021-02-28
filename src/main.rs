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
use rustyline::{error::ReadlineError, Editor};
use tokio::task;
use std::sync::{Arc, Mutex};

mod error;
mod almond_client;
use crate::almond_client::AlmondClient;
use crate::error::AlmondError;

struct MessageHandler {
}

impl almond_client::MessageHandler for MessageHandler {
    fn on_new_text_message(self : &mut MessageHandler, msg : &str) {
        println!("Almond says: {}", msg);
    }

    fn on_new_command(self : &mut MessageHandler, msg : &str) {
        println!(">> {}", msg);
    }

    fn set_expected(self : &mut MessageHandler, _expected : Option<&str>) {
        // do nothing yet
    }
}

async fn program() -> Result<(), AlmondError> {
    let handler = Box::new(MessageHandler {});
    let client = AlmondClient::new("ws://127.0.0.1:3000/api/conversation", handler).await?;

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
                    client.lock().await.send_thingtalk(&trimmed[3..]).await?;
                } else {
                    client.lock().await.send_command(&trimmed).await?;
                }
            }

            Err(ReadlineError::Interrupted) => {
                client.lock().await.close().await?;
                return Ok::<(), AlmondError>(());
            }

            Err(ReadlineError::Eof) => {
                client.lock().await.close().await?;
                return Ok::<(), AlmondError>(());
            }

            Err(err) => {
                return Err::<(), AlmondError>(From::from(err));
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), AlmondError> {
    program().await?;

    Ok(())
}
