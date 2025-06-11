// #![feature(async_fn_in_trait)]

use std::{
    borrow::Cow,
    ops::{Deref, DerefMut},
};

// Imports
pub mod client;
pub mod error;
#[cfg(feature = "prost")]
pub mod prost;
pub mod query;
pub mod raw;

// Export
#[cfg(feature = "derive")]
pub use api_builder_derive::*;
pub use client::*;
pub use query::*;

// Re-exports
pub use bytes::Bytes;
pub use http::{
    request::Builder as RequestBuilder, HeaderMap, Method, Request, Response, StatusCode,
};
use serde::de::DeserializeOwned;
pub use url::Url;

/// A macro that is similar to `vec!` but for `http::HeaderMap`s.
/// This does not check for invalid headers.
#[macro_export]
macro_rules! headermap {
    ($(($key:expr,  $value:expr)),*) => {
        {
            let mut map = ::http::HeaderMap::new();
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
            let mut map = ::http::HeaderMap::new();
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
    pub fn append<T: Into<QueryParamPairs>>(&mut self, other: T) {
        self.0.append(&mut other.into());
    }

    pub fn push<T: Into<QueryParamPair>>(&mut self, value: T) {
        self.0.push(value.into());
    }

    pub fn push_hashmap<K: Into<Cow<'static, str>>, V: Into<Cow<'static, str>>>(
        &mut self,
        name: &str,
        value: std::collections::HashMap<K, V>,
    ) {
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
    /// Ignores any errors returned by the API.
    fn ignore_errors(&self) -> bool {
        false
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
            url.push('?');
            for x in query.iter() {
                url.push_str(&x.key);
                url.push('=');
                url.push_str(&x.value);
                url.push('&');
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
    ///
    /// Defaults to using `serde_json::from_slice`.
    fn deserialize<T: serde::de::DeserializeOwned>(
        &self,
        response: http::Response<Bytes>,
    ) -> Result<T, error::BodyError> {
        Ok(serde_json::from_slice(response.body())?)
    }
}

/// A helper trait for implementing `Query` for sync clients.
///
/// If using a combinator, make sure to implement [`Deref`](std::ops::Deref) for the combinator so the methods of the endpoint can be accessed.
#[macro_export]
macro_rules! impl_query {
    ("request") => {
        fn request(
            &self,
            client: &C,
        ) -> Result<$crate::RequestBuilder, $crate::error::APIError<C::Error>> {
            let method = self.method();
            let url = client.rest_endpoint(&self.url())?;
            let request = $crate::Request::builder()
                .method(method)
                .uri(url.to_string());
            if let Some(headers) = self.headers()? {
                let mut request = request;
                let headers_mut = request.headers_mut();
                if let Some(headers_mut) = headers_mut {
                    headers_mut.extend(headers);
                } else {
                    for (key, value) in headers {
                        request = request.header(
                            key.ok_or($crate::error::APIError::MissingHeaderName)?,
                            value,
                        );
                    }
                };
                Ok(request)
            } else {
                Ok(request)
            }
        }
    };
    ("send") => {
        fn send(
            &self,
            client: &C,
            request: $crate::RequestBuilder,
        ) -> Result<$crate::Response<$crate::Bytes>, $crate::error::APIError<C::Error>> {
            if let Some((mime, body)) = self.body()? {
                client.rest(
                    request
                        .header(::http::header::CONTENT_TYPE, mime)
                        .body(body)?,
                )
            } else {
                client.rest(request.body(Vec::new())?)
            }
        }
    };
    ("finalise") => {
        fn finalise(
            &self,
            response: $crate::Response<$crate::Bytes>,
        ) -> Result<T, $crate::error::APIError<C::Error>> {
            if !response.status().is_success() && !self.ignore_errors() {
                Err($crate::error::APIError::Response(response))?
            } else {
                Ok(self.deserialize(response)?)
            }
        }
    };
    ("query") => {
        fn query(&self, client: &C) -> Result<T, $crate::error::APIError<C::Error>> {
            $crate::query::Query::<T, C>::finalise(
                self,
                $crate::query::Query::<T, C>::send(
                    self,
                    client,
                    $crate::query::Query::<T, C>::request(self, client)?,
                )?,
            )
        }
    };
}

/// A helper trait for implementing `Query` for async clients.
///
/// If using a combinator, make sure to implement [`Deref`](std::ops::Deref) for the combinator so the methods of the endpoint can be accessed.
#[macro_export]
macro_rules! impl_query_async {
    ("request") => {
        async fn request_async(
            &self,
            client: &C,
        ) -> Result<$crate::RequestBuilder, $crate::error::APIError<C::Error>> {
            let method = self.method();
            let url = client.rest_endpoint(&self.url())?;
            let request = ::http::Request::builder()
                .method(method)
                .uri(url.to_string());
            if let Some(headers) = self.headers()? {
                let mut request = request;
                let headers_mut = request.headers_mut();
                if let Some(headers_mut) = headers_mut {
                    headers_mut.extend(headers);
                } else {
                    for (key, value) in headers {
                        request = request.header(
                            key.ok_or($crate::error::APIError::MissingHeaderName)?,
                            value,
                        );
                    }
                };
                Ok(request)
            } else {
                Ok(request)
            }
        }
    };
    ("send") => {
        async fn send_async(
            &self,
            client: &C,
            request: $crate::RequestBuilder,
        ) -> Result<$crate::Response<$crate::Bytes>, $crate::error::APIError<C::Error>> {
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
    };
    ("finalise") => {
        async fn finalise_async(
            &self,
            response: $crate::Response<$crate::Bytes>,
        ) -> Result<T, $crate::error::APIError<C::Error>> {
            if !response.status().is_success() && !self.ignore_errors() {
                Err($crate::error::APIError::Response(response))?
            } else {
                Ok(self.deserialize(response)?)
            }
        }
    };
    ("query") => {
        async fn query_async(&self, client: &C) -> Result<T, $crate::error::APIError<C::Error>> {
            $crate::query::AsyncQuery::<T, C>::finalise_async(
                self,
                $crate::query::AsyncQuery::<T, C>::send_async(
                    self,
                    client,
                    $crate::query::AsyncQuery::<T, C>::request_async(self, client).await?,
                )
                .await?,
            )
            .await
        }
    };
}

impl<E, T, C> query::Query<T, C> for E
where
    E: Endpoint,
    T: DeserializeOwned,
    C: client::Client,
{
    impl_query!("request");
    impl_query!("send");
    impl_query!("finalise");
    impl_query!("query");
}

impl<E, T, C> query::AsyncQuery<T, C> for E
where
    E: Endpoint + Sync,
    T: DeserializeOwned,
    C: client::AsyncClient + Sync,
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
                    request = request
                        .header(key.ok_or(crate::error::APIError::MissingHeaderName)?, value);
                }
            };
            Ok(request)
        } else {
            Ok(request)
        }
    }

    async fn finalise_async(
        &self,
        response: crate::Response<crate::Bytes>,
    ) -> Result<T, crate::error::APIError<C::Error>> {
        if !response.status().is_success() && !self.ignore_errors() {
            Err(crate::error::APIError::Response(response))?
        } else {
            Ok(self.deserialize(response)?)
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
}
