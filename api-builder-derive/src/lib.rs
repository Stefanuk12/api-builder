// Dependencies
use darling::{ast::NestedMeta, FromMeta};
use proc_macro::{Span, TokenStream};
use quote::quote;
use syn::{
    parse::{Error, ParseStream},
    parse_macro_input, Attribute, Ident, ItemImpl, Token,
};

/// All of the arguments that can be passed to the `api_endpoint` macro.
#[derive(Debug, FromMeta)]
struct APIEndpointArgs {
    /// The HTTP method to use.
    method: Option<Ident>,
    /// The endpoint's path.
    path: Option<syn::Expr>,
    /// Serialize the struct as the body, assumes JSON.
    /// This value specifies the content type.
    self_as_body: Option<String>,
    /// Serialize the struct as the body, assumes content-type header is `application/protobuf`.
    prost_self_as_body: Option<syn::Type>,
    /// Deserialize the response as Protobuf.
    prost_response: Option<bool>,
    /// Whether to ignore errors from the response.
    ignore_errors: Option<bool>,
}

/// All of the arguments that can be passed to the `api_rest_client` macro.
#[derive(Debug, FromMeta)]
struct APIRestClientArgs {
    error: Option<Ident>,
    base: Option<syn::Expr>,
}

/// Inspired from `async-trait`
struct ParseItemImpl(ItemImpl);
impl syn::parse::Parse for ParseItemImpl {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![impl]) {
            let mut item: ItemImpl = input.parse()?;
            if item.trait_.is_none() {
                return Err(Error::new(
                    Span::call_site().into(),
                    "expected a trait impl",
                ));
            }
            item.attrs = attrs;
            Ok(Self(item))
        } else {
            Err(lookahead.error())
        }
    }
}

/// A helper macro for adding an item to an impl block.
macro_rules! add_impl_input {
    ($input:ident, $item:ident) => {
        if let Some(item) = $item {
            $input.0.items.push(syn::ImplItem::Verbatim(item));
        }
    };
}

/// Add some methods to your impl block.
#[proc_macro_attribute]
pub fn api_endpoint(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the input
    let attr_args = match NestedMeta::parse_meta_list(args.into()) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(darling::Error::from(e).write_errors());
        }
    };
    let _args = match APIEndpointArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };
    let mut impl_input = parse_macro_input!(input as ParseItemImpl);

    // The implementation for each
    let method = _args.method.map(|m| {
        quote! {
            fn method(&self) -> api_builder::Method {
                api_builder::Method::#m
            }
        }
    });
    add_impl_input!(impl_input, method);

    let ignore_errors = _args.ignore_errors.map(|e| {
        quote! {
            fn ignore_errors(&self) -> bool {
                #e
            }
        }
    });
    add_impl_input!(impl_input, ignore_errors);

    let path = _args.path.map(|p| {
        // Check if it's a string literal
        if let syn::Expr::Lit(lit) = &p {
            if let syn::Lit::Str(s) = &lit.lit {
                return quote! {
                    fn path(&self) -> ::std::borrow::Cow<'static, str> {
                        ::std::borrow::Cow::Borrowed(#s)
                    }
                };
            }
        }

        // Additonal checks could be added like checking for constants or something...

        return quote! {
            fn path(&self) -> ::std::borrow::Cow<'static, str> {
                ::std::borrow::Cow::Owned(#p)
            }
        };
    });
    add_impl_input!(impl_input, path);

    let body = _args.self_as_body
        .map(|p| quote ! {
            fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, api_builder::error::BodyError> {
                Ok(Some((#p, ::serde_json::to_vec(self)?)))
            }
        })
        .or_else(|| _args.prost_self_as_body.map(|p| quote ! {
            fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, api_builder::error::BodyError> {
                use prost::Message;
                Ok(Some(("application/protobuf", #p::from(self.clone()).encode_to_vec())))
            }
        }));
    add_impl_input!(impl_input, body);

    if _args.prost_response.unwrap_or_default() {
        impl_input.0.items.push(syn::ImplItem::Verbatim(quote! {
            fn deserialize(&self, response: api_builder::Response<api_builder::Bytes>) -> Result<Self::Response, api_builder::error::BodyError> {
                use prost::Message;
                Self::Response::decode(response.body().clone()).map_err(|_| api_builder::error::BodyError::Deserialize)
            }
        }));
    }

    // Return the input
    let inner_impl = impl_input.0;
    TokenStream::from(quote!(#inner_impl))
}

/// Implements `RestClient`.
#[proc_macro_attribute]
pub fn api_rest_client(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the input
    let attr_args = match NestedMeta::parse_meta_list(args.into()) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(darling::Error::from(e).write_errors());
        }
    };
    let _args = match APIRestClientArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };
    let mut impl_input = parse_macro_input!(input as ParseItemImpl);

    // The implementation for each
    let error = _args.error.map(|e| {
        quote! {
            type Error = #e;
        }
    });
    add_impl_input!(impl_input, error);

    let base = _args.base
        .map(|b| quote! {
            fn rest_endpoint(&self, path: &str) -> Result<api_builder::Url, api_builder::error::APIError<Self::Error>> {
                let url = api_builder::Url::parse(#b).unwrap();
                Ok(url.join(path).unwrap())
            }
        });
    add_impl_input!(impl_input, base);

    // Return the input
    let inner_impl = impl_input.0;
    TokenStream::from(quote!(#inner_impl))
}

/// Implements `ReqwestClient`, assumes that the struct has a `client` field.
#[proc_macro_derive(ReqwestClient)]
pub fn derive_reqwest_client(input: TokenStream) -> TokenStream {
    // Parse the input
    let input = parse_macro_input!(input as syn::ItemStruct);
    let name = input.ident;

    // Return the input
    TokenStream::from(quote! {
        impl api_builder::client::ReqwestClient for #name {
            fn client(&self) -> &::reqwest::blocking::Client {
                &self.client
            }
        }
    })
}

/// Implements `ReqwestAsyncClient`, assumes that the struct has a `async_client` field.
#[proc_macro_derive(ReqwestAsyncClient)]
pub fn derive_reqwest_async_client(input: TokenStream) -> TokenStream {
    // Parse the input
    let input = parse_macro_input!(input as syn::ItemStruct);
    let name = input.ident;

    // Return the input
    TokenStream::from(quote! {
        impl api_builder::client::ReqwestAsyncClient for #name {
            fn client(&self) -> &::reqwest::Client {
                &self.async_client
            }
        }
    })
}
