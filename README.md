# pocketbase-rs

A *still in development* Rust API Wrapper to interact with the [PocketBase Web API](https://pocketbase.io/). 

## Note

This project is still under development, the different API methods are added when they are needed for an external project.   
Pull Requests are welcome, whether it's about adding missing methods, adding and fixing documentation, and more.   

This crate was last tested on `PocketBase` version `0.23.7`.

## Installation

Simply add this line in your `cargo.toml` file:

```toml
[dependencies]
pocketbase-rs = { git = "https://github.com/fromhorizons/pocketbase-rs", rev="commit-hash-here" }
```

Since the crate is not currently published to [crates.io](https://crates.io), it's hosted directly here on GitHub. Using the `rev` field ensures that your project depends on a specific commit of the repository. This is important because any breaking change merged into the main branch could break your code the next time you use the `cargo update` command.   

__Replace the `commit-hash-here` with the specific hash you want to depend on.__

## Usage

The different methods are generally named the same as the official JavaScript SDK. Usage of this Rust crate is usually similiar to it.

```rust
use std::Error;

use pocketbase_rs::PocketBase;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct Article {
  name: String,
  content: String,
}

#[tokio::main]
 async fn main() -> Result<(), Error> {
  let mut pb = PocketBase::new("http://localhost:8081");

  // Authenticate the new client

  let auth_data = pb
      .collection("_superusers")
      .auth_with_password("user@domain.com", "secure-password")
      .await?;

  // Create record

  let new_record = pb
      .collection("articles")
      .create::<Article>(Article {
          name: "Vulpes Vulpes".to_string(),
          content: "The red fox (Vulpes vulpes) is the largest of the true foxes and one of the most widely distributed members. [source: Wikipedia, the free encyclopedia]".to_string(),
      })
      .await?;

  println!("Created article: {:?}", record);

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

## Licence

This project is free and open source. All code in this repository is dual-licensed under either:

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or
  <http://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Rust by Example by you, as defined in the Apache-2.0 license, shall be dually licensed as above, without any additional terms or conditions.
