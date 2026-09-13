// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::pin::Pin;

use async_stream::stream;
use futures_util::Stream;
use opentalk_database::DatabaseError;
use snafu::ResultExt;

use crate::error::DatabaseSnafu;

pub(crate) async fn convert_db_stream_to_inventory_stream<'a, TIn: 'a, TOut: 'a>(
    stream: impl Stream<Item = Result<TIn, DatabaseError>> + 'a,
    item_mapper: fn(TIn) -> TOut,
) -> Result<Pin<Box<dyn Stream<Item = opentalk_inventory_common::Result<TOut>> + 'a>>, DatabaseError>
{
    let stream = stream! {
        for await item in stream {
            let item = item.context(DatabaseSnafu)?;
            yield Ok(item_mapper(item));
        }
    };
    Ok(Box::pin(stream))
}
