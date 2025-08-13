//! OpenVM to-rocq tool for pretty printing AIR circuits
//!
//! This crate provides functionality to inspect and format OpenVM AIR circuits
//! in various output formats including text, JSON, and Rocq-compatible formats.

pub(crate) mod circuit_printer;
pub(crate) mod commands;

pub use commands::print_circuit;
