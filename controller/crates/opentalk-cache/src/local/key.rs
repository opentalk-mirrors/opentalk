// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::hash::Hash;

pub trait Key: Hash + Eq + Send + Sync {}

impl<T: Hash + Eq + Send + Sync> Key for T {}
