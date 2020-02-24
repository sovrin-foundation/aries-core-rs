/*
 * Copyright 2020
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * -----------------------------------------------------------------------------
 */
#![deny(
    warnings,
    missing_docs,
    unsafe_code,
    unused_import_braces,
    unused_qualifications
)]
//! The core functionality for implementing an Aries agent.
//!
//! The core functionality consists of three components
//!
//! 1. Frontend - This is used to communicate to the agent
//! 2. Backend  - This is used to communicate to the system behind the agent
//! 3. Storage  - This is used by the agent to store information that is only accessible by the agent
//!
//! Each of these is pluggable and can be used to create your own custom agent.
//!
//! ## Frontend
//!
//! The frontend protocol my use HTTP as its protocol and is the default. It opens an HTTP port and then
//! switches to use websockets. Another frontend could use Noise sockets or another protocol.
//! Each must be configurable via the `--features=` option at compile time. Multiple backends may be
//! used.
//!
//! ## Backend
//!
//! Backend system represent resources to which the agent will act as the entry point.
//! For example, an agent could be used to talk to the Indy or Sawtooth blockchains which use ZMQ.
//! Again, this must be configurable via the `--features=` option at compile time.
//!
//! ## Storage
//!
//! The storage layer is how the agent will store state and other information long term.
//! This should be separate from the backend system but doesn't have to be.
//! Again, this must be configurable via the `--features=` option at compile time.
//! The focus of this project is to enable secure, misuse-resistant agent storage.
//! 
//! Storage is composed of the authentication layer
//! the data protection layer, and data persistance layer.
//!
//! Authentication is how a connection to the storage is
//! created and authorized.
//!
//! Data protection handles encrypting/decrypting data
//! depending on how the system is to be architected.
//! This layer could be bundled with authentication and/or
//! the persistance layer. This is decoupled to allow
//! for better flexibility
//!
//! Data persistance is just putting the data where it can
//! be retrieved later. This could be a database, files,
//! cloud storage, or memory.

#[macro_use] extern crate bitflags;

/// The security modules
pub mod security;
/// The persistence modules
pub mod persistence;

extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
