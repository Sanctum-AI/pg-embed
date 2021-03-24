//!
//! # pg-embed
//!
//! [![Crates.io](https://img.shields.io/crates/v/pg-embed)](http://crates.io/crates/pg-embed)
//! [![Docs.rs](https://docs.rs/pg-embed/badge.svg)](https://docs.rs/pg-embed)
//! [![Crates.io](https://img.shields.io/crates/d/pg-embed)](http://crates.io/crates/pg-embed)
//! [![Crates.io](https://img.shields.io/crates/l/pg-embed)](https://github.com/faokunega/pg-embed/blob/master/LICENSE)
//!
//! Run a Postgresql database locally on Linux, MacOS or Windows as part of another Rust application or test.
//!
//! # Usage
//!
//! - Add pg-embed & zip to your Cargo.toml
//!
//! ```
//! [dependencies]
//! pg-embed = "0.2"
//! zip = "0.5.11"
//! ```
//!
//! A postgresql instance can be created using<br/>
//! **[PgEmbed]( postgres::PgEmbed )::new([PgSettings]( postgres::PgSettings ), [FetchSettings]( fetch::FetchSettings ))**
//!
//! # Examples
//!
//! ```
//! use pg_embed::postgres::{PgEmbed, PgSettings};
//! use pg_embed::fetch;
//! use pg_embed::fetch::{OperationSystem, Architecture, FetchSettings, PG_V13};
//!
//! let pg_settings = PgSettings{
//!     /// where to store the postgresql executables
//!     executables_dir: "data/postgres".to_string(),
//!     /// where to store the postgresql database
//!     database_dir: "data/db".to_string(),
//!     port: 5432,
//!     user: "postgres".to_string(),
//!     password: "password".to_string(),
//!     /// if persistent is false clean up files and directories on drop, otherwise keep them
//!     persistent: false
//! };
//! let fetch_settings = FetchSettings{
//!     host: "https://repo1.maven.org".to_string(),
//!     operating_system: OperationSystem::Darwin,
//!     architecture: Architecture::Amd64,
//!     version: PG_V13
//! };
//! let mut pg_emb = PgEmbed::new(pg_settings, fetch_settings);
//!
//! /// async block only to show that these methods need to be executed in an async context
//! async {
//!
//!     /// Download, unpack, create password file and database
//!     pg_emb.setup().await;
//!
//!     /// start postgresql database
//!     pg_emb.start_db().await;
//!
//!     /// stop postgresql database
//!     pg_emb.stop_db().await;
//! };
//!
//!
//! ```
//!
//! # Notes
//!
//! Reliant on the great work being done by [zonkyio/embedded-postgres-binaries](https://github.com/zonkyio/embedded-postgres-binaries) in order to fetch precompiled binaries from [Maven](https://mvnrepository.com/artifact/io.zonky.test.postgres/embedded-postgres-binaries-bom).
//!
//! ## License
//!
//! pg-embed is licensed under the MIT license. Please read the [LICENSE-MIT](https://github.com/faokunega/pg-embed/blob/master/LICENSE) file in this repository for more information.
//!
//! ## Recent Breaking Changes
//!
//! pg-embed follows semantic versioning, so breaking changes should only happen upon major version bumps. The only exception to this rule is breaking changes that happen due to implementation that was deemed to be a bug, security concerns, or it can be reasonably proved to affect no code. For the full details, see [CHANGELOG.md](https://github.com/faokunega/pg-embed/blob/master/CHANGELOG.md).
//!
pub mod fetch;
pub mod postgres;
pub mod errors;

