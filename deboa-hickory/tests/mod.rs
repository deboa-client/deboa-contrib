#![allow(unused)]
use deboa::{dns::DnsResolver as _, request::get};
use deboa_extras::serde::json::JsonBody;
use deboa_hickory::HickoryDnsResolver;
use deboa_tokio::CustomClient;
use hickory_resolver::{
    Hosts, TokioResolver,
    config::{GOOGLE, ResolverConfig, ResolverOpts},
    net::runtime::TokioRuntimeProvider,
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

#[tokio::test]
async fn test_lookup() -> Result<(), Box<dyn std::error::Error>> {
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
