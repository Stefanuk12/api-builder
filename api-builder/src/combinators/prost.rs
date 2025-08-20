use bytes::Bytes;
use http::Response;
use prost::Message;

use crate::{
    async_queryer,
    error::{APIError, APIErrorKind, BodyError},
    queryer, AsyncClient, AsyncQuery, Client, Endpoint, Query,
};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Prost<E>(pub E);
impl<E> std::ops::Deref for Prost<E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<E, T, C> Query<T, C> for Prost<E>
where
    E: Endpoint,
    T: Message + Default,
    C: Client,
{
    queryer!("request");
    queryer!("send");
    queryer!("query");

    fn finalise(&self, response: Response<Bytes>) -> Result<T, APIError<C::Error>> {
        if !response.status().is_success() && !self.0.ignore_errors() {
            Err(APIErrorKind::Response(response))?
        } else {
            // Deserialize the response
            Ok(T::decode(response.body().clone())
                .map_err(|_| APIErrorKind::Body(BodyError::Deserialize))?)
        }
    }
}

impl<E, T, C> AsyncQuery<T, C> for Prost<E>
where
    E: Endpoint + Sync,
    T: Message + Default,
    C: AsyncClient + Sync,
{
    async_queryer!("request");
    async_queryer!("query");
    async_queryer!("send");

    async fn finalise_async(&self, response: Response<Bytes>) -> Result<T, APIError<C::Error>> {
        if !response.status().is_success() && !self.0.ignore_errors() {
            Err(APIErrorKind::Response(response))?
        } else {
            // Deserialize the response
            Ok(T::decode(response.into_body())
                .map_err(|_| APIErrorKind::Body(BodyError::Deserialize))?)
        }
    }
}
