# API Builder

This crate aims to make it easy to build API bindings in Rust.
It is inspired by [gitlab](https://plume.benboeckel.net/~/JustAnotherBlog/designing-rust-bindings-for-rest-ap-is) and how they build their API bindings.

For some examples, especially on the macros, view [examples](./api-builder/examples/).

## The ideology

The contents of the entire HTTP request, minus authentication (see [handling authentication](#handling-authentication)), is within a single endpoint struct.
From there, users can supply their own client, response type, and other handlers via combinators.
Therefore, your bindings will not lock users to a specific HTTP client and can customise the client and response to fit their needs.

## The benefits

- It's more "rusty"
- Users can use both **async** and **synchronous** methods
- Easy testing via mock client implementations
- Customisable behaviour
  - Custom response type
  - Bring Your Own HTTP Client
  - Supports "middleware" via combinators and custom clients

## Handling authentication

Authentication is sensitive and if the bindings you make are supposed to be consumed directly by the user, you might not want them to supply their API token each time.
While that approach is perfectly fine, it can be tedious and the behaviour can be abstracted to your own client which handles it all.

For example, you can make your own client skeleton which stores the API token and a user-supplied sub client.
From there, you implement [`RestClient`](https://github.com/Stefanuk12/api-builder/blob/master/api-builder/src/client/mod.rs#L16) and forward that implementation to the inner sub client.
However, you add custom logic to [`Client::rest`](https://github.com/Stefanuk12/api-builder/blob/master/api-builder/src/client/mod.rs#L29)/[`AsyncClient::rest_async`](https://github.com/Stefanuk12/api-builder/blob/master/api-builder/src/client/mod.rs#L36) which would add the corrosponding authentication headers.
Now, you can remove the API token from the endpoint struct and force users to use your client skeleton.

Alternatively, you can make your own combinator which takes in the endpoint and any tokens.
The combinator approach might make it easier when these tokens are directly present in the body of the request, but you don't want to include them in the endpoint struct.
However, this shifts the original problem to another place, and that's why I prefer the client route.

For a full example of the client approach, you can read [luarmor-rs](https://github.com/Stefanuk12/luarmor-rs/tree/master/src).

## Additional advice

- Make sure to respect [`Endpoint::ignore_errors`](https://github.com/Stefanuk12/api-builder/blob/master/api-builder/src/endpoint.rs#L12) within your [`Query`](https://github.com/Stefanuk12/api-builder/blob/master/api-builder/src/query.rs#L8)/[`AsyncQuery`](https://github.com/Stefanuk12/api-builder/blob/master/api-builder/src/query.rs#L26) implementations
- Use [`typed_builder`](https://docs.rs/typed-builder/latest/typed_builder/derive.TypedBuilder.html) on your endpoint structs to make them easier to construct
- Implement [`APIClientError`](https://github.com/Stefanuk12/api-builder/blob/main/api-builder/src/error/mod.rs#L46) on your custom client errors to get the `From<E>` (and `Try`) impl
