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
//! Errors are represented as a kind of error.
//!
//! `EnclaveErrorKind` is used to represent this.
//! `EnclaveError` represents messages as strings that are attached to a kind of error as well
//! as the context in which the error is thrown and a backtrace.
use failure::{Backtrace, Context, Fail};
use std::fmt;

/// Enclave Errors from failures
#[derive(Clone, Debug, Eq, PartialEq, Fail)]
pub enum EnclaveErrorKind {
    /// Occurs when a connection cannot be made to the enclave
    #[fail(display = "An error occurred while connecting to the enclave: {}", msg)]
    ConnectionFailure {
        /// Description of what failed during the connection
        msg: String,
    },
    /// Occurs when the incorrect credentials are supplied to the enclave
    #[fail(display = "Access was denied to the enclave: {}", msg)]
    AccessDenied {
        /// Description of how access was denied
        msg: String,
    },
    /// When a item in the enclave is does not exist
    #[fail(display = "The specified item is not found in the keyring")]
    ItemNotFound,
    /// Catch all if currently not handled or doesn't meet another error category like a general message
    #[fail(display = "{}", msg)]
    GeneralError {
        /// Generic message
        msg: String,
    },
}

/// Wrapper class for `EnclaveErrorKind`, `Backtrace`, and `Context`
#[derive(Debug)]
pub struct EnclaveError {
    /// The error kind that occurred
    inner: Context<EnclaveErrorKind>,
}

impl EnclaveError {
    /// Create from a message and kind
    pub fn from_msg<D: fmt::Display + fmt::Debug + Send + Sync + 'static>(
        kind: EnclaveErrorKind,
        msg: D,
    ) -> Self {
        Self {
            inner: Context::new(msg).context(kind),
        }
    }

    /// Extract the internal error kind
    pub fn kind(&self) -> EnclaveErrorKind {
        self.inner.get_context().clone()
    }
}

impl From<EnclaveErrorKind> for EnclaveError {
    fn from(e: EnclaveErrorKind) -> Self {
        Self {
            inner: Context::new("").context(e),
        }
    }
}

impl Fail for EnclaveError {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl fmt::Display for EnclaveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut first = true;

        for cause in Fail::iter_chain(&self.inner) {
            if first {
                first = false;
                writeln!(f, "Error: {}", cause)?;
            } else {
                writeln!(f, "Caused by: {}", cause)?;
            }
        }
        Ok(())
    }
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
impl From<security_framework::base::Error> for EnclaveError {
    fn from(e: security_framework::base::Error) -> Self {
        match e.code() {
            -128 => EnclaveErrorKind::AccessDenied {
                msg: format!("{:?}", e.to_string()),
            }
            .into(),
            -25300 => EnclaveErrorKind::ItemNotFound.into(),
            _ => EnclaveErrorKind::GeneralError {
                msg: "Unknown error".to_string(),
            }
            .into(),
        }
    }
}
