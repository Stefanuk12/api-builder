/// A macro that is similar to [vec!] but for [http::HeaderMap]s.
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

/// A macro that is similar to [vec!] but for [http::HeaderMap]s.
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

/// A helper trait for implementing [Query](crate::Query) for sync clients.
///
/// If using a combinator, make sure to implement [`Deref`](core::ops::Deref) for the combinator so the methods of the endpoint can be accessed.
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
                            key.ok_or($crate::error::HeaderError::MissingHeaderName)?,
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
                        .header(::http::header::CONTENT_TYPE, mime.as_ref())
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
                Err($crate::error::APIErrorKind::Response(response))?
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
pub(crate) use impl_query as queryer;

/// A helper trait for implementing [AsyncQuery](crate::AsyncQuery) for async clients.
///
/// If using a combinator, make sure to implement [`Deref`](core::ops::Deref) for the combinator so the methods of the endpoint can be accessed.
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
                            key.ok_or($crate::error::HeaderError::MissingHeaderName)?,
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
                            .header(
                                ::http::header::CONTENT_TYPE,
                                ::http::header::HeaderValue::from_str(&mime)?,
                            )
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
                Err($crate::error::APIErrorKind::Response(response))?
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
pub(crate) use impl_query_async as async_queryer;
