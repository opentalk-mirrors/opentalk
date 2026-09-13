// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

pub trait Value: Clone + Send + Sync {}

impl<T: Clone + Send + Sync> Value for T {}
