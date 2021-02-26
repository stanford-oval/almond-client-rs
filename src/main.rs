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
use futures::stream::TryStreamExt;

async fn program() -> Result<(), Box<dyn Error>> {
    let (mut client, _) = tokio_tungstenite::connect_async("ws://127.0.0.1:3000/api/conversation").await?;

    let done = client.try_for_each(|msg| async move {
        println!("{:#?}", msg);
        Ok(())
    });

    done.await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    program().await?;

    Ok(())
}
