use bytes::Bytes;
use prost::Message;

use crate::{
    error::APIError, impl_query, AsyncClient, AsyncQuery, Client, Endpoint, Query,
};

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
    impl_query!("request");
    impl_query!("send");
    impl_query!("query");

    fn finalise(&self, response: http::Response<Bytes>) -> Result<T, APIError<C::Error>> {
        if !response.status().is_success() && !self.0.ignore_errors() {
            Err(APIError::Response(response))?
        } else {
            // Deserialize the response
            Ok(T::decode(response.body().clone())
                .map_err(|_| APIError::Body(crate::error::BodyError::Deserialize))?)
        }
    }
}

#[async_trait::async_trait(?Send)]
impl<E, T, C> AsyncQuery<T, C> for Prost<E>
where
    E: Endpoint + Sync,
    T: Message + Default,
    C: AsyncClient + Sync,
{
    // impl_query_async!("request");
    async fn request_async(
        &self,
        client: &C,
    ) -> Result<crate::RequestBuilder, crate::error::APIError<C::Error>> {
        let method = self.method();
        let url = client.rest_endpoint(&self.url())?;
        let request = http::Request::builder().method(method).uri(url.to_string());
        if let Some(headers) = self.headers()? {
            let mut request = request;
            let headers_mut = request.headers_mut();
            if let Some(headers_mut) = headers_mut {
                headers_mut.extend(headers);
            } else {
                for (key, value) in headers {
                    request = request.header(
                        key.ok_or(crate::error::APIError::MissingHeaderName)?,
                        value,
                    );
                }
            };
            Ok(request)
        } else {
            Ok(request)
        }
    }
        
    async fn query_async(&self, client: &C) -> Result<T, crate::error::APIError<C::Error>> {
        crate::query::AsyncQuery::<T, C>::finalise_async(
            self,
            crate::query::AsyncQuery::<T, C>::send_async(
                self,
                client,
                crate::query::AsyncQuery::<T, C>::request_async(self, client).await?,
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
    ) -> Result<T, crate::error::APIError<C::Error>> {
        if !response.status().is_success() && !self.0.ignore_errors() {
            Err(APIError::Response(response))?
        } else {
            // Deserialize the response
            Ok(T::decode(response.body().clone())
                .map_err(|_| APIError::Body(crate::error::BodyError::Deserialize))?)
        }
    }
}
