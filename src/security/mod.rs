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
//! Security can be managed by two parts: hardware enclaves or software enclaves.
//!
//! Enclaves are assumed to be specialized crypto modules, usually in hardware,
//! that have been audited for compliance and security. These should be used
//! for key storage and crypto operations to better protect against side channel
//! attacks and key extraction methods. The downside to hardware enclaves is portability.
//! Private keys usually cannot be removed from the enclave and thus presents a
//! problem for backup and recovery. Keys that need to backup and recovery
//! should not be stored solely in hardware enclaves. Instead, use the hardware to
//! wrap/unwrap those keys.
//!
//! Software enclaves usually do not provide the same guarantees as hardware but
//! have the flexibility of portability and deployment. The best approach is use
//! a combination of these two to create an optimal solution.
//!
//! For example, use the software enclave provided by the operating system to
//! store credentials to the hardware or external enclave. Once the credentials
//! are retrieved from the OS enclave, they can be used to connect to the
//! hardware or external enclave.

use std::{fmt, path::Path};
use zeroize::Zeroize;

/// Typical result from performing and enclave operation or sending an enclave message
pub type EnclaveResult<T> = Result<T, errors::EnclaveError>;
/// Configuration options for connecting to Secure Enclaves
///
/// Each enclave has its own unique configuration requirements
/// but are wrapped by this config to enable generic interfaces
///
/// Enclaves are meant for secure handling of keys. Some enclaves
/// support more crypto primitives like encryption and signatures.
/// For now, we do not support attestations as these are often
/// broken anyway and complex.
#[derive(Debug)]
pub enum EnclaveConnector<A, B>
where
    A: AsRef<Path>,
    B: Into<String>,
{
    /// Connect to an instance of an OsKeyRing
    OsKeyRing(OsKeyRingConnector<A, B>),
    /// Connect to a Yubihsm
    YubiHsm,
}

impl<A, B> fmt::Display for EnclaveConnector<A, B>
where
    A: AsRef<Path>,
    B: Into<String>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EnclaveConfig ({})", self)
    }
}

/// Configuration options for connecting to the OS Keying which
/// may be backed by a hardware enclave
#[derive(Clone, Debug, PartialEq, Eq, Zeroize)]
pub struct OsKeyRingConnector<A: AsRef<Path>, B: Into<String>> {
    /// Path to the keyring. If `None`, it will use the default OS keyring
    path: Option<A>,
    /// The username to use for logging in. If `None`, the user will be prompted
    username: Option<B>,
    /// The password to use for logging in. If `None`, the user will be prompted
    password: Option<B>,
}

impl<A, B> fmt::Display for OsKeyRingConnector<A, B>
where
    A: AsRef<Path>,
    B: Into<String>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "OsKeyRingConfig (path: {:?}, username: {:?}, password: {:?})",
            self.path.as_ref().map(|p| p.as_ref().as_os_str()),
            self.username.as_ref().map(|_| "*********"),
            self.password.as_ref().map(|_| "*********")
        )
    }
}

/// All enclaves structs should use this trait so the callers
/// can simply use them without diving into the details
/// for each unique configuration. This trait is meant
/// to be used by the non-security minded and should be hard
/// to mess up––misuse resistant.
pub trait EnclaveLike: Sized {
    /// Establish a connection to the enclave
    fn connect<A: AsRef<Path>, B: Into<String>>(
        config: EnclaveConnector<A, B>,
    ) -> EnclaveResult<Self>;
    /// Close the connection to the enclave
    fn close(self);
    /// The capabilities of the enclave
    fn capabilities(&self) -> EnclaveCapabilities;
}

/// Valid key types that can be created in an enclave.
///
/// Not all enclaves support all key types. Please review
/// the documentation for your respective enclave to know
/// each of their capabilities.
#[derive(Clone, Copy, Debug)]
pub enum EnclaveKeyType {
    /// Twisted Edwards signing key
    Ed25519,
    /// Key-exchange key on Curve25519
    X25519,
    /// Elliptic Curve diffie-hellman key-exchange key
    Ecdh(EcCurves),
    /// Elliptic Curve signing key
    Ecdsa(EcCurves, EcdsaAlgorithm),
    /// RSA encryption key with Optimal Asymmetric Encryption Padding
    RsaOaep(RsaMgf),
    /// RSA signing key legacy algorithm using PCKS#1v1.5 signatures (ASN.1 DER encoded)
    /// Strongly consider using ECDSA or ED25519 or RSAPSS instead
    RsaPkcs15(RsaMgf),
    /// RSASSA-PSS: Probabilistic Signature Scheme based on the RSASP1 and RSAVP1 primitives with the EMSA-PSS encoding method.
    RsaPss(RsaMgf),
    /// Key for use with Hash-based Message Authentication Code tags
    Hmac(HmacAlgorithm),
    /// Key for encrypting/decrypting data
    WrapKey(WrappingKey),
}

/// Valid algorithms for wrapping data
#[derive(Clone, Copy, Debug)]
pub enum WrappingKey {
    /// AES encryption algorithm
    Aes(AesSizes, AesModes),
    /// XChachaPoly1305 encryption algorithm
    XChaChaPoly1305,
}

/// Valid sizes for the AES algorithm
#[derive(Clone, Copy, Debug)]
pub enum AesSizes {
    /// AES with 128 bit keys
    Aes128,
    /// AES with 192 bit keys
    Aes192,
    /// AES with 256 bit keys
    Aes256,
}

/// Valid AEAD modes for AES
#[derive(Clone, Copy, Debug)]
pub enum AesModes {
    /// Counter with CBC-MAC mode. This is a NIST approved mode of operation defined in SP 800-38C
    Ccm,
    /// Galios Counter mode. This is a NIST approved mode of operation defined in SP 800-38C
    Gcm,
    /// Galios Counter mode with Synthetic IV as defined in RFC8452
    GcmSiv,
}

/// Valid curves for ECC operations
#[derive(Clone, Copy, Debug)]
pub enum EcCurves {
    /// NIST P-256 curve
    Secp256r1,
    /// NIST P-384 curve
    Secp384r1,
    /// NIST P-512 curve
    Secp512r1,
    /// Koblitz 256 curve
    Secp256k1,
}

/// Valid algorithms for ECDSA signatures
#[derive(Clone, Copy, Debug)]
pub enum EcdsaAlgorithm {
    /// Sign/Verify ECC signatures using SHA1
    /// Only use for legacy purposes as SHA1 is considered broken
    Sha1,
    /// Sign/Verify ECC signatures using SHA2-256
    Sha256,
    /// Sign/Verify ECC signatures using SHA2-384
    Sha384,
    /// Sign/Verify ECC signatures using SHA2-512
    Sha512,
}

/// Valid algorithms for HMAC keys
#[derive(Clone, Copy, Debug)]
pub enum HmacAlgorithm {
    /// Sign/Verify HMAC tags using SHA1
    /// Only use for legacy purposes as SHA1 is considered broken
    Sha1,
    /// Sign/Verify HMAC tags using SHA2-256
    Sha256,
    /// Sign/Verify HMAC tags using SHA2-384
    Sha384,
    /// Sign/Verify HMAC tags using SHA2-512
    Sha512,
}

/// Mask generating functions for RSA signatures
#[derive(Clone, Copy, Debug)]
pub enum RsaMgf {
    /// Sign/Verify RSA signatures using SHA1
    /// Only use for legacy purposes as SHA1 is considered broken
    Sha1,
    /// Sign/Verify RSA signatures using SHA2-256
    Sha256,
    /// Sign/Verify RSA signatures using SHA2-384
    Sha384,
    /// Sign/Verify RSA signatures using SHA2-512
    Sha512,
}

bitflags! {
    /// All Enclave Capabilities
    pub struct EnclaveCapabilities: u64 {
        /// Can compute elliptic curve diffie-hellman
        const DERIVE_ECDH                      = 0x0000_0000_0000_0001;
        /// Can compute diffie-hellman using x25519
        const DERIVE_X25519                    = 0x0000_0000_0000_0002;
        /// Can generate RSA-OAEP keys
        const GENERATE_OAEP_KEY                = 0x0000_0000_0000_0004;
        /// Can generate RSA-PSS keys
        const GENERATE_PSS_KEY                 = 0x0000_0000_0000_0008;
        /// Can generate RSA-PKCS1v1.5 keys
        const GENERATE_PKCS_KEY                = 0x0000_0000_0000_0010;
        /// Can generate keys used for AES encryption
        const GENERATE_AES_KEY                 = 0x0000_0000_0000_0020;
        /// Can generate keys used for computing HMACs
        const GENERATE_HMAC_KEY                = 0x0000_0000_0000_0040;
        /// Can generate keys used for XChaCha20Poly1305 encryption
        const GENERATE_XCHACHA20_POLY1305_KEY  = 0x0000_0000_0000_0080;
        /// Can generate keys used for generating ECDSA signatures
        const GENERATE_ECDSA_KEY               = 0x0000_0000_0000_0100;
        /// Can generate keys used for generating EDDSA signatures
        const GENERATE_EDDSA_KEY               = 0x0000_0000_0000_0200;
        /// Can generate random data
        const GENERATE_RANDOM                  = 0x0000_0000_0000_0400;
        /// Can sign data using the RSA-PSS algorithm
        const SIGN_PSS                         = 0x0000_0000_0000_0800;
        /// Can sign data with the RSA-PKCS1v1.5 algorithm
        const SIGN_PKCS                        = 0x0000_0000_0000_1000;
        /// Can sign data with ECDSA
        const SIGN_ECDSA                       = 0x0000_0000_0000_2000;
        /// Can sign data with EDDSA
        const SIGN_EDDSA                       = 0x0000_0000_0000_4000;
        /// Can sign data with an HMAC
        const SIGN_HMAC                        = 0x0000_0000_0000_8000;
        /// Can verify an ECDSA signature
        const VERIFY_ECDSA                     = 0x0000_0000_0001_0000;
        /// Can verify an EDDSA signature
        const VERIFY_EDDSA                     = 0x0000_0000_0002_0000;
        /// Can verify an HMAC
        const VERIFY_HMAC                      = 0x0000_0000_0004_0000;
        /// Can verify an RSA-PSS signature
        const VERIFY_PSS                       = 0x0000_0000_0008_0000;
        /// Can verify an RSA-PKCS1v1.5 signature
        const VERIFY_PKCS                      = 0x0000_0000_0010_0000;
        /// Can delete an RSA-OAEP key
        const DELETE_OAEP_KEY                  = 0x0000_0000_0020_0000;
        /// Can delete an RSA-PSS key
        const DELETE_PSS_KEY                   = 0x0000_0000_0040_0000;
        /// Can delete an RSA-PKCS1v1.5 key
        const DELETE_PCKS_KEY                  = 0x0000_0000_0080_0000;
        /// Can delete an AES key
        const DELETE_AES_KEY                   = 0x0000_0000_0100_0000;
        /// Can delete an HMAC key
        const DELETE_HMAC_KEY                  = 0x0000_0000_0200_0000;
        /// Can delete an XChaCha20Poly1305 key
        const DELETE_XCHACHA20_POLY1305        = 0x0000_0000_0400_0000;
        /// Can wrap keys
        const WRAP_KEY                         = 0x0000_0000_0800_0000;
        /// Can unwrap keys
        const UNWRAP_KEY                       = 0x0000_0000_1000_0000;
        /// Can export wrapped keys
        const EXPORT_WRAPPED_KEY               = 0x0000_0000_2000_0000;
        /// Can import wrapped keys
        const IMPORT_WRAPPED_KEY               = 0x0000_0000_4000_0000;
        /// Can save an RSA-OAEP key that is not wrapped
        const PUT_OAEP_KEY                     = 0x0000_0000_8000_0000;
        /// Can save an RSA-PSS key that is not wrapped
        const PUT_PSS_KEY                      = 0x0000_0001_0000_0000;
        /// Can save an RSA-PKCS1v1.5 key that is not wrapped
        const PUT_PKCS_KEY                     = 0x0000_0002_0000_0000;
        /// Can save an AES key that is not wrapped
        const PUT_AES_KEY                      = 0x0000_0004_0000_0000;
        /// Can save an HMAC key that is not wrapped
        const PUT_HMAC_KEY                     = 0x0000_0008_0000_0000;
        /// Can save an XChaCha20Poly1305 key that is not wrapped
        const PUT_XCHACHA20_POLY1305_KEY       = 0x0000_0010_0000_0000;
        /// Can save an ECDSA key that is not wrapped
        const PUT_ECDSA_KEY                    = 0x0000_0020_0000_0000;
        /// Can save an EDDSA key that is not wrapped
        const PUT_EDDSA_KEY                    = 0x0000_0040_0000_0000;
        /// Can encrypt data using an RSA-OAEP public key
        const ENCRYPT_OAEP                     = 0x0000_0080_0000_0000;
        /// Can encrypt data using an RSA-PKCS1v1.5 public key
        const ENCRYPT_PKCS                     = 0x0000_0100_0000_0000;
        /// Can encrypt data using AES symmetric key
        const ENCRYPT_AES                      = 0x0000_0200_0000_0000;
        /// Can encrypt data using XChaCha20Poly1305 symmetric key
        const ENCRYPT_XCHACHA20_POLY1305       = 0x0000_0400_0000_0000;
        /// Can decrypt data using RSA-OAEP private key
        const DECRYPT_OAEP                     = 0x0000_0800_0000_0000;
        /// Can decrypt data using RSA-PKCS1v1.5 private key
        const DECRYPT_PKCS                     = 0x0000_1000_0000_0000;
        /// Can decrypt data using AES symmetric key
        const DECRYPT_AES                      = 0x0000_2000_0000_0000;
        /// Can decrypt data using XChaCha20Poly1305 symmetric key
        const DECRYPT_XCHACHA20_POLY1305       = 0x0000_4000_0000_0000;
    }
}

bitflags! {
    /// All capabilities supported by symmetric keys
    pub struct SymmetricCapability: u16 {
        /// Encrypt data using a symmetric algorithm
        const ENCRYPT                 = 0x0000_0001;
        /// Decrypt data using a symmetric algorithm
        const DECRYPT                 = 0x0000_0002;
        /// Compute the Hash-based Message-Authentication-Code over data
        const HMAC_SIGN               = 0x0000_0004;
        /// Verify the Hash-based Message-Authentication-Code over data
        const HMAC_VERIFY             = 0x0000_0008;
        /// Export a symmetric key that is wrapped by another key
        const EXPORT_WRAPPED          = 0x0000_0010;
        /// Import a symmetric key that is wrapped by another key
        const IMPORT_WRAPPED          = 0x0000_0020;
        /// Allow symmetric key to be exported if wrapped/encrypted
        const EXPORTABLE_WHEN_WRAPPED = 0x0000_0100;
    }
}

bitflags! {
    /// All capabilities of Ecc keys
    pub struct EccCapability: u16 {
        /// Sign data using ECDSA/EDDSA private key
        const SIGN                    = 0x0000_0001;
        /// Verify signature using ECDSA/EDDSA public key
        const VERIFY                  = 0x0000_0002;
        /// Compute diffie hellman secret with ECDH/x25519
        const DERIVE_DIFFIE_HELLMAN   = 0x0000_0004;
        /// Allow ECC key to exported if wrapped/encrypted
        const EXPORTABLE_WHEN_WRAPPED = 0x0000_0100;
    }
}

bitflags! {
    /// All capabilities of RSA keys
    pub struct RsaCapability: u16 {
        /// Encrypt data using RSA-OAEP public key
        const ENCRYPT_OAEP            = 0x0000_0001;
        /// Decrypt data using RSA-OAEP private key
        const DECRYPT_OAEP            = 0x0000_0002;
        /// Sign data using RSA-OAEP private key
        const SIGN_PSS                = 0x0000_0004;
        /// Verify signature using RSA-OAEP public key
        const VERIFY_PSS              = 0x0000_0008;
        /// Sign using RSA-PKCS1v1.5 private key
        const SIGN_PKCS               = 0x0000_0010;
        /// Verify signature using RSA-PKCS1v1.5 public key
        const VERIFY_PKCS             = 0x0000_0020;
        /// Allow RSA key to exported if it is wrapped/encrypted
        const EXPORTABLE_WHEN_WRAPPED = 0x0000_0100;
    }
}

/// Provides access to the OS keyring and enclaves
pub mod os;

/// A null enclave. Basically is just a pass through.
///
/// Do NOT use this except for debugging purposes or
/// your backend already provides crypto services
pub mod null;

/// Errors that can occur for Enclave operations
pub mod errors;
