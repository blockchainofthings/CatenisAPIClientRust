# Catenis API Client for Rust

This library is used to make it easier to access the Catenis API services from code
written in the Rust programming language.

> **Note**: this release of the library targets **version 0.12** of the Catenis API.

## Documentation

The complete library documentation can be found [here](https://docs.rs/catenis_api_client/~2.0).

## Usage

To start using the library, one needs to instantiate a new [`CatenisClient`](https://docs.rs/catenis_api_client/~2.0/catenis_api_client/struct.CatenisClient.html)
object. Then, to make a call to an API method, just call the corresponding method on the client object.

### Example

```rust
use catenis_api_client::{
    CatenisClient, ClientOptions, Environment, Result,
};

// Instantiate Catenis API client object
let mut ctn_client = CatenisClient::new_with_options(
    Some((
        "drc3XdxNtzoucpw9xiRp",
        concat!(
            "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0",
            "d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3"
        ),
    ).into()),
    &[
        ClientOptions::Environment(Environment::Sandbox),
    ],
)?;

// Call Read Message API method
let result = ctn_client.read_message("o3muoTnnD6cXYyarYY38", None)?;

println!("Read message result: {:?}", result);
```

## Notification

The library also makes it easy for receiving notifications from the Catenis system through its
[`WsNotifyChannel`](https://docs.rs/catenis_api_client/~2.0/catenis_api_client/notification/struct.WsNotifyChannel.html)
data structure, which embeds a WebSocket client.

## Asynchronous processing

The library allows for asynchronous processing using the [Tokio](https://crates.io/crates/tokio/0.2.24)
runtime.

> **Note**: only Tokio version 0.2 is currently supported.

To activate asynchronous processing, the **`async`** feature must be enabled.

```toml
catenis_api_client = { version = "2.0", features = ["async"] }
```

The asynchronous version of the client can then be accessed from the [`async_impl`](https://docs.rs/catenis_api_client/~2.0/catenis_api_client/async_impl/index.html)
module.

### Example

```rust
use catenis_api_client::{
    async_impl,
    ClientOptions, Environment, Result,
};

// Instantiate asynchronous Catenis API client object
let mut ctn_client = async_impl::CatenisClient::new_with_options(
    Some((
        "drc3XdxNtzoucpw9xiRp",
        concat!(
            "4c1749c8e86f65e0a73e5fb19f2aa9e74a716bc22d7956bf3072b4bc3fbfe2a0",
            "d138ad0d4bcfee251e4e5f54d6e92b8fd4eb36958a7aeaeeb51e8d2fcc4552c3"
        ),
    ).into()),
    &[
        ClientOptions::Environment(Environment::Sandbox),
    ],
)?;
```

## Catenis API Documentation

For further information on the Catenis API, please reference the [Catenis API Documentation](https://catenis.com/docs/api).

## License

This library is distributed under the terms of both the [MIT License](LICENSE-MIT) and the [Apache License (Version 2.0)](LICENSE-APACHE).

Copyright © 2021-2022, Blockchain of Things Inc.