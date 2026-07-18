//! mortar-core: everything mason's feed engine needs, compiled two ways -
//! into the native axum server (mortar-server) and into a browser service
//! worker (mortar-wasm). Platform differences live in `platform`.

pub mod algo;
pub mod cache;
pub mod config;
pub mod error;
pub mod feed;
pub mod fixtures;
pub mod http;
pub mod mode;
pub mod model;
pub mod persist;
pub mod platform;
pub mod sources;
pub mod state;
