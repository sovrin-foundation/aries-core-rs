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
//! Keychain wrapper for interacting with the Apple Secure Enclave.
//! The Keychain Service API requires signed code to access much of its
//! functionality. Accessing many APIs from an unsigned app will return
//! an error with a kind of `ErrorKind::MissingEntitlement`.
//! //! Follow the instructions here to create a self-signed code signing certificate:
//! <https://developer.apple.com/library/archive/documentation/Security/Conceptual/CodeSigningGuide/Procedures/Procedures.html>
//!
//! You will need to use the [codesign] command-line utility (or XCode) to sign
//! your code before it will be able to access most Keychain Services API
//! functionality. When you sign, you will need an entitlements file which
//! grants access to the Keychain Services API. Below is an example:
//!
//! ```xml
//! <?xml version="1.0" encoding="UTF-8"?>
//! <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
//! <plist version="1.0">
//! <dict>
//!	<key>keychain-access-groups</key>
//!	<array>
//!		<string>$(AppIdentifierPrefix)com.example.MyApplication</string>
//!	</array>
//! </dict>
//! </plist>
//! ```
//!
//! [codesign]: https://developer.apple.com/library/archive/documentation/Security/Conceptual/CodeSigningGuide/Procedures/Procedures.html#//apple_ref/doc/uid/TP40005929-CH4-SW4

use super::super::{EnclaveConfig, EnclaveErrorKind, EnclaveLike, EnclaveResult};

//use keychain_services::keychain::KeyChain;
use security_framework::os::macos::keychain::*;
use std::path::Path;

/// MacOSX and iOS implementation for keyrings that function like an enclave
pub struct MacOsKeyRing(SecKeychain);

impl MacOsKeyRing {
    /// Create a new keyring at the specified location on the filesystem
    pub fn create(path: &Path, password: Option<&str>) -> EnclaveResult<Self> {
        let mut options = CreateOptions::new();
        options.prompt_user(false);
        match password {
            Some(p) => options.password(p),
            None => options.prompt_user(true),
        };

        Ok(Self(options.create(path)?))
    }

    /// Open a keyring from a file
    pub fn open(path: &Path) -> EnclaveResult<Self> {
        Ok(Self(SecKeychain::open(path)?))
    }

    /// Unlock the keyring. If `password` is not specified, the user will be prompted to enter it.
    pub fn unlock(&mut self, password: Option<&str>) -> EnclaveResult<()> {
        self.0.unlock(password)?;
        Ok(())
    }

    /// Connect to the user's default keyring
    pub fn default() -> EnclaveResult<Self> {
        let keychain = SecKeychain::default()?;
        Ok(Self(keychain))
    }
}

impl EnclaveLike for MacOsKeyRing {
    fn connect<A: AsRef<Path>, B: Into<String>>(
        config: EnclaveConfig<A, B>,
    ) -> EnclaveResult<Self> {
        if let EnclaveConfig::OsKeyRing(c) = config {
            let pass = c.password.map(|p| p.into());
            let mut keychain = match c.path {
                Some(p) => {
                    let path = p.as_ref();
                    if path.exists() {
                        Self::create(path, pass.as_ref().map(|e| e.as_str()))?
                    } else {
                        Self::open(path)?
                    }
                }
                None => Self::default()?,
            };
            keychain.unlock(pass.as_ref().map(|p| p.as_str()))?;
            Ok(keychain)
        } else {
            Err(EnclaveErrorKind::ConnectionFailure {
                msg: format!(
                    "Invalid configuration type. Expected OsKeyRing but found {}",
                    config
                ),
            }
            .into())
        }
    }

    fn close(self) {}
}
