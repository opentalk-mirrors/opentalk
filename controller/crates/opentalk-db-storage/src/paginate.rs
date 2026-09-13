// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::pagination::{Page, PageSize};

use crate::paginated::Paginated;

/// Pagination trait for diesel
pub(crate) trait Paginate: Sized {
    fn paginate_by(self, per_page: PageSize, page: Page) -> Paginated<Self>;
}

impl<T> Paginate for T {
    fn paginate_by(self, per_page: PageSize, page: Page) -> Paginated<Self> {
        Paginated {
            query: self,
            per_page,
            offset: per_page.first_index_on_page_saturating(page),
        }
    }
}
