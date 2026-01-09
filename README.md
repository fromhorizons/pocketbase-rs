# pocketbase-rs

A Rust wrapper around [PocketBase Rest API](https://pocketbase.io/)'s REST API.

## Usage

Most of the methods in this SDK are named and organized as closely as possible to the official PocketBase SDK. Using this Rust crate is generally similar.

```rust
use std::error::Error;

use pocketbase_rs::PocketBase;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct Article {
  name: String,
  content: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
  let mut pb = PocketBase::new("http://localhost:8090");

  // Authenticate the new client
  let auth_data = pb
      .collection("users")
      .auth_with_password("YOUR_EMAIL_OR_USERNAME", "YOUR_PASSWORD")
      .await?;

  // Create new record
  let new_record = pb
      .collection("articles")
      .create::<Article>(Article {
          name: "Vulpes Vulpes".to_string(),
          content: "The red fox (Vulpes vulpes) is the largest of the true foxes and one of the most widely distributed members. [source: Wikipedia, the free encyclopedia]".to_string(),
      })
      .await?;

  println!("Created article: {:?}", new_record);

  // Get records list

  let records = pb
      .collection("articles")
      .get_list::<Article>()
      .sort("-created,id")
      .call()
      .await?;

  for record in records.items {
    println!("{record:?}");
  }

  Ok(())
}
```

## Note

Not all SDK features are implemented yet and are generally added when needed for other projects.  
PRs aimed at adding these missing features, as well as other additions and fixes, are more than welcome.

This crate was last tested on `PocketBase` version `0.34.2`.

## Licence

This project is free and open source. All code in this repository is dual-licensed under either:

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  <http://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
