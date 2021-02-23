#![feature(
    const_fn,
    decl_macro,
    format_args_capture,
    proc_macro_hygiene,
)]

mod cache;
mod cli;
mod entity;
mod error;
mod parse;
mod providers;
mod server;
mod schema;
mod query;

use crate::cli::CliArgs;
use crate::providers::{FsProvider, FsProviderConfig};
use crate::server::{Server, ServerConfig};

fn main() {
    let args = CliArgs::from_cli();

    let server = Server::new(ServerConfig {
        bind_address: args.address,
        port: args.port,
    });

    let provider = FsProvider::new(FsProviderConfig {
        root: args.content_path,
    });

    server.listen(provider);
}
