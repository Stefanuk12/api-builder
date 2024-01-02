use bytes::Bytes;
use prost::Message;

use crate::{Endpoint, Client, Query, error::APIError, impl_query, AsyncQuery, AsyncClient, impl_query_async};

pub struct Prost<E> {
    endpoint: E
}
impl<E> std::ops::Deref for Prost<E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.endpoint
    }
}
impl<E, T, C> Query<T, C> for Prost<E>
where
    E: Endpoint,
    T: Message + Default,
    C: Client,
{
    impl_query!("request");
    impl_query!("send");
    impl_query!("query");

    fn finalise(&self, response: http::Response<Bytes>) -> Result<T, APIError<C::Error>> {
        if !response.status().is_success() && !self.endpoint.ignore_errors() {
            Err(APIError::Response(response))?
        } else {
            // Deserialize the response
            Ok(T::decode(response.body().clone()).map_err(|_| APIError::Body(crate::error::BodyError::Deserialize))?)
        }
    }
}

impl<E, T, C> AsyncQuery<T, C> for Prost<E>
where
    E: Endpoint + Sync,
    T: Message + Default,
    C: AsyncClient + Sync,
{
    impl_query_async!("request");
    impl_query_async!("send");
    impl_query_async!("query");

    async fn finalise_async(
        &self,
        response: ::http::Response<::bytes::Bytes>,
    ) -> Result<T, crate::error::APIError<C::Error>> {
        if !response.status().is_success() && !self.endpoint.ignore_errors() {
            Err(APIError::Response(response))?
        } else {
            // Deserialize the response
            Ok(T::decode(response.body().clone()).map_err(|_| APIError::Body(crate::error::BodyError::Deserialize))?)
        }
    }
}