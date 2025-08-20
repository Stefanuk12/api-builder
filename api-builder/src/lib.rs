macro_rules! import {
    ($($module:ident),* $(,)?) => {
        $(
            pub mod $module;
            #[allow(unused_imports)]
            pub use $module::*;
        )*
    };
}

// Imports
pub mod client;
pub mod error;
pub mod query;

import!(combinators);
import!(endpoint);
import!(macros);
import!(query_params);

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
pub use serde_json;
pub use url::Url;
