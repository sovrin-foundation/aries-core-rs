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
use failure::{Backtrace, Context, Fail};
use std::fmt;

/// Represents possible errors that could occur in the persistence layer.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum PersistenceErrorKind {
    /// Occurs when a bad config setting is used
    #[fail(display = "An invalid configuration was supplied")]
    InvalidConfig,
    /// Occurs during an IO error
    #[fail(display = "IO Error")]
    IOError,
    #[fail(display = "Success")]
    Success,
    #[fail(display = "could not write or find db")]
    DBError
}

/// Represents a Persistence error that includes a context and backtrace
#[derive(Debug)]
pub struct PersistenceError {
    inner: Context<PersistenceErrorKind>
}

impl PersistenceError {
    /// Get `PersistenceErrorKind` wrapped by this error
    pub fn kind(&self) -> PersistenceErrorKind {
        self.inner.get_context().clone()
    }
}

impl From<PersistenceErrorKind> for PersistenceError {
    fn from(kind: PersistenceErrorKind) -> Self {
        Self {
            inner: Context::new("").context(kind)
        }
    }
}

impl From<Context<PersistenceErrorKind>> for PersistenceError {
    fn from(inner: Context<PersistenceErrorKind>) -> Self {
        PersistenceError { inner }
    }
}

impl Fail for PersistenceError {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl fmt::Display for PersistenceError {
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

