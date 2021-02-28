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

use tokio_tungstenite::tungstenite;
use tokio::task::JoinError;
use rustyline::error::ReadlineError;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum AlmondError {
    WebSocketError(tungstenite::Error),
    JSONError(serde_json::Error),
    JoinError(JoinError),
    ReadlineError(ReadlineError),
    ProtocolError(&'static str)
}

impl fmt::Display for AlmondError {
    fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match &self {
            AlmondError::WebSocketError(err) => {
                write!(f, "web socket error: {}", err)
            }

            AlmondError::JSONError(err) => {
                write!(f, "json/protocol error: {}", err)
            }

            AlmondError::JoinError(err) => {
                write!(f, "task completion error: {}", err)
            }

            AlmondError::ReadlineError(err) => {
                write!(f, "readline error: {}", err)
            }

            AlmondError::ProtocolError(err) => {
                write!(f, "protocol error: {}", err)
            }
        }
    }
}

impl From<tungstenite::Error> for AlmondError {
    fn from(err : tungstenite::Error) -> AlmondError {
        AlmondError::WebSocketError(err)
    }
}

impl From<serde_json::Error> for AlmondError {
    fn from(err : serde_json::Error) -> AlmondError {
        AlmondError::JSONError(err)
    }
}

impl From<JoinError> for AlmondError {
    fn from(err : JoinError) -> AlmondError {
        AlmondError::JoinError(err)
    }
}

impl From<ReadlineError> for AlmondError {
    fn from(err : ReadlineError) -> AlmondError {
        AlmondError::ReadlineError(err)
    }
}

impl Error for AlmondError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self {
            AlmondError::WebSocketError(err) => {
                Some(err)
            }

            AlmondError::JSONError(err) => {
                Some(err)
            }

            AlmondError::JoinError(err) => {
                Some(err)
            }

            AlmondError::ReadlineError(err) => {
                Some(err)
            }

            _ => {
                None
            }
        }
    }
}
