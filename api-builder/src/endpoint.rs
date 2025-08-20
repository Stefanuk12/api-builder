use std::borrow::Cow;

use bytes::Bytes;
use http::{HeaderMap, Method, Response};
use serde::de::DeserializeOwned;

use crate::{
    error::{BodyError, HeaderError},
    QueryParamPairs,
};

/// A trait for providing the necessary information for a single REST API endpoint
pub trait Endpoint {
    /// Ignores any errors returned by the API.
    fn ignore_errors(&self) -> bool {
        false
    }

    /// The method for the endpoint.
    fn method(&self) -> Method {
        Method::GET
    }

    /// The path for the endpoint.
    fn path(&self) -> Cow<'static, str>;

    /// Any additional headers for the endpoint.
    fn headers(&self) -> Result<Option<HeaderMap>, HeaderError> {
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
    fn body(&self) -> Result<Option<(Cow<'static, str>, Vec<u8>)>, BodyError> {
        Ok(None)
    }

    /// Deserialize the response bytes.
    ///
    /// Defaults to using `serde_json::from_slice`.
    fn deserialize<T: DeserializeOwned>(&self, response: Response<Bytes>) -> Result<T, BodyError> {
        Ok(serde_json::from_slice(response.body())?)
    }
}
