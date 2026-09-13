// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use core::future::Future;

use crate::{Error, Inventory, Result};

/// Helper trait to work around rustc limitations with `AsyncFnOnce`.
///
/// This is needed to assert that the future returned by the async closure is `Send`,
/// while maintaining working type inference. See diesel-async's `AsyncFunc` for the
/// upstream pattern this mirrors.
pub trait AsyncFunc<T, R>:
    AsyncFnOnce(T) -> R + FnOnce(T) -> <Self as AsyncFunc<T, R>>::Fut
{
    type Fut: Future<Output = R>;
}

impl<F, T, Fut, R> AsyncFunc<T, R> for F
where
    F: AsyncFnOnce(T) -> R + FnOnce(T) -> Fut,
    Fut: Future<Output = R>,
{
    type Fut = Fut;
}

/// Executes the given function inside of a database transaction
#[expect(clippy::manual_async_fn)] // Follows the upstream implementation. Implementing this as async fn triggers a rustc bug.
pub fn transaction<'a, 'inv, I, F, R, E>(
    inventory: &'inv mut I,
    callback: F,
) -> impl Future<Output = Result<R, E>> + Send + 'inv
where
    I: Inventory + ?Sized,
    for<'r> F: AsyncFnOnce(&'r mut I) -> Result<R, E>
        + AsyncFunc<&'r mut I, Result<R, E>, Fut: Send>
        + Send
        + 'a,
    E: From<Error> + Send,
    R: Send,
    'a: 'inv,
{
    async move {
        let callback = callback;

        inventory.begin_transaction().await?;
        match callback(&mut *inventory).await {
            Ok(value) => {
                inventory.commit_transaction().await?;
                Ok(value)
            }
            Err(user_error) => match inventory.rollback_transaction().await {
                Ok(()) => Err(user_error),
                Err(Error::BrokenTransactionManager) => {
                    // In this case we are probably more interested by the
                    // original error, which likely caused this
                    Err(user_error)
                }
                Err(rollback_error) => Err(rollback_error.into()),
            },
        }
    }
}
