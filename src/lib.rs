// SPDX-License-Identifier: AGPL-3.0-or-later
//! `quire-semantic-value`: a shared `no_std` leaf crate above the `quire-exact`
//! kernel.
//!
//! It holds the runtime semantic values that a compiler, an evaluator and a
//! backend share, so the code exists once. Its dependencies are the
//! `quire-exact` kernel, the one RFC 8785 encoder (`quire-canonical`, built
//! without `std`), `serde` and `thiserror`. It uses only `core` and `alloc`.
#![no_std]

extern crate alloc;
#[cfg(test)]
extern crate std;

pub mod call;
pub mod checking;
pub mod containment;
pub mod declaration;
pub mod definition;
pub mod enumeration;
pub mod location;
pub mod loss;
pub mod object_closure;
pub mod quantity;
pub mod semantic_node;
pub mod stop;
pub mod unit;
