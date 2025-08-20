// This isn't a combinator, but just the default behaviour and implementation for `Query` and `AsyncQuery`.

use serde::de::DeserializeOwned;

use crate::{AsyncClient, AsyncQuery, Client, Endpoint, Query, async_queryer, queryer};

impl<E, T, C> Query<T, C> for E
where
    E: Endpoint,
    T: DeserializeOwned,
    C: Client,
{
    queryer!("request");
    queryer!("send");
    queryer!("finalise");
    queryer!("query");
}

impl<E, T, C> AsyncQuery<T, C> for E
where
    E: Endpoint + Sync,
    T: DeserializeOwned,
    C: AsyncClient + Sync,
{
    async_queryer!("request");
    async_queryer!("send");
    async_queryer!("query");
    async_queryer!("finalise");
}
