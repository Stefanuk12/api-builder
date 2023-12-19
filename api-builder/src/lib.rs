use std::{borrow::Cow, ops::{Deref, DerefMut}};

// Dependencies
use error::APIError;

// Imports
pub mod client;
pub mod error;
pub mod query;

// Export
#[cfg(feature = "derive")]
pub use api_builder_derive::*;
pub use client::*;
pub use query::*;

// Re-exports
pub use http::{HeaderMap, Method, request::Builder as RequestBuilder, StatusCode, Response};
pub use url::Url;
pub use bytes::Bytes;
pub use async_trait::async_trait;

/// A macro that is similar to `vec!` but for `http::HeaderMap`s.
/// This does not check for invalid headers.
#[macro_export]
macro_rules! headermap {
    ($(($key:expr,  $value:expr)),*) => {
        {
            let mut map = ::api_builder::HeaderMap::new();
            $(
                map.insert($key, $value.parse().unwrap());
            )*
            map
        }
    };
}

/// A macro that is similar to `vec!` but for `http::HeaderMap`s.
/// This does check for invalid headers.
#[macro_export]
macro_rules! headermap_checked {
    ($(($key:expr,  $value:expr)),*) => {
        {
            let mut map = ::api_builder::HeaderMap::new();
            $(
                map.insert($key, $value.parse()?);
            )*
            map
        }
    };
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct QueryParamPair {
    pub key: Cow<'static, str>,
    pub value: Cow<'static, str>,
}
impl QueryParamPair {
    pub fn new<K, V>(key: K, value: V) -> Self
    where
        K: Into<Cow<'static, str>>,
        V: Into<Cow<'static, str>>,
    {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}
impl<K: Into<Cow<'static, str>>, V: Into<Cow<'static, str>>> From<(K, V)> for QueryParamPair {
    fn from(pair: (K, V)) -> Self {
        Self {
            key: pair.0.into(),
            value: pair.1.into(),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct QueryParamPairs(pub Vec<QueryParamPair>);
impl QueryParamPairs {
    pub fn push<T: Into<QueryParamPair>>(&mut self, value: T) {
        self.0.push(value.into());
    }

    pub fn push_hashmap<K: Into<Cow<'static, str>>, V: Into<Cow<'static, str>>>(&mut self, name: &str, value: std::collections::HashMap<K, V>) {
        for (key, value) in value {
            let key: Cow<'_, _> = key.into();
            self.push((format!("{}[{}]", name, key), value));
        }
    }
}
impl Deref for QueryParamPairs {
    type Target = Vec<QueryParamPair>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for QueryParamPairs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl From<Vec<QueryParamPair>> for QueryParamPairs {
    fn from(pairs: Vec<QueryParamPair>) -> Self {
        Self(pairs)
    }
}
impl<K: Into<Cow<'static, str>>, V: Into<Cow<'static, str>>> From<Vec<(K, V)>> for QueryParamPairs {
    fn from(value: Vec<(K, V)>) -> Self {
        Self(value.into_iter().map(|x| x.into()).collect())
    }
}

/// A trait for providing the necessary information for a single REST API endpoint
pub trait Endpoint {
    /// The response type for the endpoint.
    type Response;

    /// Whether to ignore errors from response.
    fn ignore_errors(&self) -> bool {
        false
    }

    /// Maps a response to an error.
    fn map_error<C: RestClient>(&self, response: http::Response<Bytes>) -> APIError<C::Error> {
        APIError::Response(response)
    }

    /// The method for the endpoint.
    fn method(&self) -> http::Method {
        http::Method::GET
    }

    /// The path for the endpoint.
    fn path(&self) -> std::borrow::Cow<'static, str>;

    /// Any additional headers for the endpoint.
    fn headers(&self) -> Result<Option<http::HeaderMap>, error::HeaderError> {
        Ok(None)
    }

    /// The query parameters for the endpoint.
    fn query_params(&self) -> Option<QueryParamPairs> {
        None
    }

    /// Builds the full URL, including query.
    fn url(&self) -> String {
        let mut url = self.path().to_string();
        if let Some(query) = self.query_params() {
            url.push_str("?");
            for x in query.iter() {
                url.push_str(&x.key);
                url.push_str("=");
                url.push_str(&x.value);
                url.push_str("&");
            }
            url.pop();
        }
        url
    }

    /// The body for the endpoint.
    /// 
    /// Returns the `Content-Encoding` header for the data as well as the data itself.
    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, error::BodyError> {
        Ok(None)
    }

    /// Deserialize the response bytes.
    fn deserialize(&self, response: http::Response<Bytes>) -> Result<Self::Response, error::BodyError>;
}

impl<E, T, C> query::Query<T, C> for E
where
    E: Endpoint<Response = T>,
    C: client::Client,
{
    fn request(&self, client: &C) -> Result<http::request::Builder, APIError<C::Error>> {
        // Build the URL
        let method = self.method();
        let url = client.rest_endpoint(&self.url())?;

        // Build the request
        let request = http::Request::builder()
            .method(method)
            .uri(url.to_string());

        // Add the headers
        if let Some(headers) = self.headers()? {
            let mut request = request;
            let headers_mut = request.headers_mut();

            if let Some(headers_mut) = headers_mut {
                headers_mut.extend(headers);
            } else {
                for (key, value) in headers {
                    request = request.header(key.ok_or(APIError::MissingHeaderName)?, value);
                }
            };
            Ok(request)
        } else {
            Ok(request)
        }
    }

    fn send(&self, client: &C, request: http::request::Builder) -> Result<http::Response<Bytes>, APIError<C::Error>> {
        if let Some((encoding, body)) = self.body()? {
            client.rest(
                request
                    .header(http::header::CONTENT_TYPE, encoding)
                    .body(body)?
            )
        } else {
            client.rest(
                request
                    .body(vec![])?
            )
        }
    }

    fn finalise(&self, response: http::Response<Bytes>) -> Result<T, APIError<C::Error>> {
        if !response.status().is_success() && !self.ignore_errors() {
            Err(self.map_error::<C>(response))?
        } else {
            // Deserialize the response
            Ok(self.deserialize(response).or(Err(APIError::Body(error::BodyError::Deserialize)))?)
        }
    }

    fn query(&self, client: &C) -> Result<T, APIError<C::Error>> {
        query::Query::<T, C>::finalise(self,
            self.send(
                client,
                self.request(client)?
            )?
        )
    }
}

#[async_trait]
impl<'a, E, T, C> AsyncQuery<T, C> for E
where
    E: Endpoint<Response = T> + Sync,
    C: AsyncClient + Sync,
{
    async fn request_async(&self, client: &C) -> Result<http::request::Builder, APIError<C::Error>> {
        // Build the URL
        let method = self.method();
        let url = client.rest_endpoint(&self.url())?;

        // Build the request
        let request = http::Request::builder()
            .method(method)
            .uri(url.to_string());

        // Add the headers
        if let Some(headers) = self.headers()? {
            let mut request = request;
            let headers_mut = request.headers_mut();

            if let Some(headers_mut) = headers_mut {
                headers_mut.extend(headers);
            } else {
                for (key, value) in headers {
                    request = request.header(key.ok_or(APIError::MissingHeaderName)?, value);
                }
            };
            Ok(request)
        } else {
            Ok(request)
        }
    }

    async fn send_async(&self, client: &C, request: http::request::Builder) -> Result<http::Response<Bytes>, APIError<C::Error>> {
        if let Some((encoding, body)) = self.body()? {
            client.rest_async(
                request
                    .header(http::header::CONTENT_TYPE, encoding)
                    .body(body)?
            ).await
        } else {
            client.rest_async(
                request
                    .body(vec![])?
            ).await
        }
    }

    async fn finalise_async(&self, response: http::Response<Bytes>) -> Result<T, APIError<C::Error>> {
        if !response.status().is_success() && !self.ignore_errors() {
            Err(self.map_error::<C>(response))?
        } else {
            // Deserialize the response
            Ok(self.deserialize(response).or(Err(APIError::Body(error::BodyError::Deserialize)))?)
        }
    }
    
    async fn query_async(&self, client: &C) -> Result<T, APIError<C::Error>> {
        query::AsyncQuery::<T, C>::finalise_async(self,
            self.send_async(
                client,
                self.request_async(client).await?
            ).await?
        ).await
    }
}