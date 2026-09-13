// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Collection data types.

pub mod one_or_many_btree_set;
pub mod one_or_many_vec;

pub use one_or_many_btree_set::OneOrManyBTreeSet;
#[cfg(feature = "serde")]
pub use one_or_many_btree_set::one_or_many_btree_set_option;
pub use one_or_many_vec::OneOrManyVec;
#[cfg(feature = "serde")]
pub use one_or_many_vec::one_or_many_vec_option;
