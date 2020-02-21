/// Provides access to the MacOS KeyRing and Enclave
#[cfg(any(target_os = "macos", target_os = "ios"))]
pub mod macos;