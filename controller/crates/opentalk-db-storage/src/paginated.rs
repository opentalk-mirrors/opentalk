// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use diesel::{
    QueryResult,
    pg::Pg,
    query_builder::{AstPass, Query, QueryFragment},
    sql_types::BigInt,
};
use diesel_async::{AsyncConnection, methods::LoadQuery};
use opentalk_types_common::pagination::{ItemCount, ItemIndex, PageSize};

/// Paginated diesel database response
#[derive(Debug, Clone, Copy, QueryId)]
pub(crate) struct Paginated<T> {
    pub(super) query: T,
    pub(super) per_page: PageSize,
    // We need to store the offset instead of the page due to
    // lifetime requirements in `QueryFragment::walk_ast(...)`.
    pub(super) offset: ItemIndex,
}

impl<T: Query> Paginated<T> {
    pub fn load_and_count<'query, U, Conn>(
        self,
        conn: &'query mut Conn,
    ) -> impl std::future::Future<Output = QueryResult<(Vec<U>, ItemCount)>> + Send + 'query
    where
        Self: LoadQuery<'query, Conn, (U, ItemCount)>,
        Conn: AsyncConnection + 'static,
        U: Send + 'query,
        T: 'query,
    {
        let results = {
            // When `diesel_async::RunQueryDsl` is imported globally, the call
            // to `results.first()` below will cause compiler errors because the
            // compiler mistakes it for `diesel_async::RunQueryDsl::first(…)`
            // and fails finding a trait implementation of `results` that
            // matches, so we restrict the import scope.
            use diesel_async::RunQueryDsl;
            self.load::<(U, ItemCount)>(conn)
        };
        async move {
            let results = results.await?;
            let total = results
                .first()
                .map(|x: &(U, ItemCount)| x.1)
                .unwrap_or(ItemCount::ZERO);
            let records = results.into_iter().map(|x| x.0).collect();
            Ok((records, total))
        }
    }
}

impl<T: Query> Query for Paginated<T> {
    type SqlType = (T::SqlType, BigInt);
}

impl<T> QueryFragment<Pg> for Paginated<T>
where
    T: QueryFragment<Pg>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, Pg>) -> QueryResult<()> {
        out.push_sql("SELECT *, COUNT(*) OVER () FROM (");
        self.query.walk_ast(out.reborrow())?;
        out.push_sql(") t LIMIT ");
        out.push_bind_param::<BigInt, _>(&self.per_page)?;
        out.push_sql(" OFFSET ");
        out.push_bind_param::<BigInt, _>(&self.offset)?;
        Ok(())
    }
}
