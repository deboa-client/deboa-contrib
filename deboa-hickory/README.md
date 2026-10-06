# deboa-hickory

[![Crates.io downloads](https://img.shields.io/crates/d/deboa-hickory)](https://crates.io/crates/deboa-hickory) [![crates.io](https://img.shields.io/crates/v/deboa-hickory?style=flat-square)](https://crates.io/crates/deboa-hickory) [![Build Status](https://github.com/deboa-client/deboa/actions/workflows/rust.yml/badge.svg?event=push)](https://github.com/deboa-client/deboa/actions/workflows/rust.yml) ![Crates.io MSRV](https://img.shields.io/crates/msrv/deboa-hickory) [![Documentation](https://docs.rs/deboa-hickory/badge.svg)](https://docs.rs/deboa-hickory/latest/deboa-hickory) [![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/deboa-client/deboa-contrib/blob/main/LICENSE.md) [![codecov](https://codecov.io/gh/deboa-client/deboa-contrib/graph/badge.svg?token=T0HSBAPVSI)](https://codecov.io/gh/deboa-client/deboa-contrib)

## Install

Either run from command line:

`cargo add deboa-hickory`

Or add to your `Cargo.toml`:

```toml
deboa-hickory = "0.1.1"
```

## Usage

```rust
#![allow(unused)]
use deboa::{dns::DnsResolver as _, request::get};
use deboa_extras::serde::json::JsonBody;
use deboa_hickory::HickoryDnsResolver;
use deboa_tokio::CustomClient;
use hickory_resolver::{
    config::{ResolverConfig, ResolverOpts, GOOGLE},
    net::runtime::TokioRuntimeProvider,
    Hosts, TokioResolver,
};
use http::header;
use once_cell::sync::Lazy;
use std::sync::Arc;

static GLOBAL_RESOLVER: Lazy<Arc<TokioResolver>> = Lazy::new(|| {
    let mut resolver = TokioResolver::builder_with_config(
        ResolverConfig::udp_and_tcp(&GOOGLE),
        TokioRuntimeProvider::default(),
    )
    .build()
    .expect("Failed to create Hickory resolver");

    resolver.set_hosts(Hosts::default().into());

    Arc::new(resolver)
});

#[derive(serde::Deserialize, Debug)]
struct Post {
    pub id: u32,
    pub title: String,
    pub body: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new Client instance, set timeouts, catches and protocol.
    let dns_resolver = HickoryDnsResolver::new(GLOBAL_RESOLVER.clone());
    let client = CustomClient::<HickoryDnsResolver<TokioRuntimeProvider>>::builder()
        .dns_resolver(dns_resolver)
        .build();

    let posts: Vec<Post> = get("https://jsonplaceholder.typicode.com/posts")?
        .header(header::CONTENT_TYPE, "application/json")?
        .send_with(&client)
        .await?
        .body_as(JsonBody)
        .await?;

    assert_eq!(posts.len(), 100);

    Ok(())
}
```

## License

Licensed under either of

- Apache License, Version 2.0
  (LICENSE-APACHE or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license
  (LICENSE-MIT or <https://opensource.org/licenses/MIT>)

at your option.

## Author

Rogerio Pereira Araujo <rogerio.araujo@gmail.com>
