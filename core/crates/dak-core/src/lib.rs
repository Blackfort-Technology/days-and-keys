// SPDX-License-Identifier: MPL-2.0
//! `dak-core`: Kryptokern von Days & Keys (Architektur Abschnitt 3, Protokoll Teil A).
#![forbid(unsafe_op_in_unsafe_fn)]

pub mod api;
pub mod cbor;
pub mod fehler;
pub mod kern;
pub mod kette;
pub mod modell;
pub mod sicherung;
pub mod sodium;
pub mod speicher;
pub mod tresor;
pub mod vorlagen;
pub mod woerter;
pub mod zeit;
