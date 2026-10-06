# Deboa Contrib

**deboa-contrib** contains crates with additional functionality for the deboa HTTP client.

## Install

```toml
deboa-extras = { version = "0.0.9" }
```

## Usage

```rust
use deboa::{
    request::{DeboaRequest, FetchWith, get},
    Result,
};
use deboa_tokio::Client;
use deboa_extras::serde::json::JsonBody;

#[tokio::main]
async fn main() -> Result<()> {
  // Create a new Client instance, set timeouts, catches and protocol.
  let client = Client::new();

  let posts: Vec<Post> = get("https://jsonplaceholder.typicode.com/posts")?
    .header(header::CONTENT_TYPE, "application/json")
    .send_with(&client)
    .await?
    .body_as(JsonBody)
    .await?;

  println!("posts: {:#?}", posts);

  Ok(())
}
```

## Subcrates

### [deboa-extras](https://github.com/deboa-client/deboa-contrib/tree/main/deboa-extras)

Pluggable compression/decompression, serializers, sse, websockets and catchers.
All of them are optional. This is the place to contribute with your own pluggable features.

### [deboa-fory](https://github.com/deboa-client/deboa-contrib/tree/main/deboa-fory)

Data encoding and decoding using Apache Fory.

### [deboa-hickory](https://github.com/deboa-client/deboa-contrib/tree/main/deboa-hickory)

Data encoding and decoding using Apache Fory.

## License

Licensed under either of

- Apache License, Version 2.0
  (LICENSE-APACHE or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license
  (LICENSE-MIT or <https://opensource.org/licenses/MIT>)

at your option.

## Author

Rogerio Pereira Araujo <rogerio.araujo@gmail.com>
