use bytes::Bytes;
use http::Response;

use crate::{
    error::APIError, impl_query, impl_query_async, AsyncClient, AsyncQuery, Client, Endpoint, Query,
};

pub struct Raw<E>(pub E);
impl<E> std::ops::Deref for Raw<E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<E, C> Query<Response<Bytes>, C> for Raw<E>
where
    E: Endpoint,
    C: Client,
{
    impl_query!("request");
    impl_query!("send");
   
    fn query(&self, client: &C) -> Result<Response<Bytes>, crate::error::APIError<C::Error>> {
        crate::query::Query::<Response<Bytes>, C>::finalise(
            self,
            crate::query::Query::<Response<Bytes>, C>::send(
                self,
                client,
                crate::query::Query::<Response<Bytes>, C>::request(self, client)?,
            )?,
        )
    }

    fn finalise(&self, response: http::Response<Bytes>) -> Result<Response<Bytes>, APIError<C::Error>> {
        if !response.status().is_success() && !self.0.ignore_errors() {
            Err(APIError::Response(response))?
        } else {
            // Deserialize the response
            Ok(response)
        }
    }
}

#[async_trait::async_trait(?Send)]
impl<E, C> AsyncQuery<Response<Bytes>, C> for Raw<E>
where
    E: Endpoint + Sync,
    C: AsyncClient + Sync,
{
    impl_query_async!("request");
    
    async fn query_async(&self, client: &C) -> Result<Response<Bytes>, crate::error::APIError<C::Error>> {
        crate::query::AsyncQuery::<Response<Bytes>, C>::finalise_async(
            self,
            crate::query::AsyncQuery::<Response<Bytes>, C>::send_async(
                self,
                client,
                crate::query::AsyncQuery::<Response<Bytes>, C>::request_async(self, client).await?,
            )
            .await?,
        )
        .await
    }

    async fn send_async(
        &self,
        client: &C,
        request: crate::RequestBuilder,
    ) -> Result<crate::Response<crate::Bytes>, crate::error::APIError<C::Error>> {
        if let Some((mime, body)) = self.body()? {
            client
                .rest_async(
                    request
                        .header(::http::header::CONTENT_TYPE, mime)
                        .body(body)?,
                )
                .await
        } else {
            client.rest_async(request.body(Vec::new())?).await
        }
    }

    async fn finalise_async(
        &self,
        response: ::http::Response<::bytes::Bytes>,
    ) -> Result<Response<Bytes>, crate::error::APIError<C::Error>> {
        if !response.status().is_success() && !self.0.ignore_errors() {
            Err(APIError::Response(response))?
        } else {
            // Deserialize the response
            Ok(response)
        }
    }
}
