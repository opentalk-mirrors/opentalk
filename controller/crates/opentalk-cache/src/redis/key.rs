// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{fmt::Display, hash::Hash};

pub trait Key: Hash + Display + Eq + Sync {}

impl<T: Hash + Display + Eq + Sync> Key for T {}
