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
//! A null enclave
//! Do NOT use this except for debugging purposes or
//! your backend already provides crypto services

use super::*;

/// A null enclave struct. Doesn't do anything cryptographically except pass data through.
pub struct NullEnclave;

impl EnclaveLike for NullEnclave {
    fn connect<A: AsRef<Path>, B: Into<String>>(_: EnclaveConfig<A, B>) -> EnclaveResult<Self> {
        Ok(Self{})
    }

    fn close(self) {}
}