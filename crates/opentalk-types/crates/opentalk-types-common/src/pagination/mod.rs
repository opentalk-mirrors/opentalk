// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Data types for handling pagination.

mod item_count;
mod item_index;
mod page;
mod page_size;

pub use item_count::{ITEM_COUNT_DEFAULT, ITEM_COUNT_MAX, ItemCount, TryFromItemCountError};
pub use item_index::{ITEM_INDEX_DEFAULT, ITEM_INDEX_MAX, ItemIndex, TryFromItemIndexError};
pub use page::{Page, TryFromPageError};
pub use page_size::{PageSize, TryFromPageSizeError};
