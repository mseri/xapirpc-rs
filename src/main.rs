#[macro_use]
extern crate quicli;

extern crate xapirpc;
extern crate xmlrpc;

use std::process;

use quicli::prelude::*;

use xapirpc::Config;

/// Minimal xapi xmlrpc CLI client
#[derive(Debug, StructOpt)]
struct Cli {
    /// XenServer host. Can be passed with the XAPI_HOST env variable.
    #[structopt(long = "host", short = "h")]
    host: Option<String>,
    /// XenServer host user name. Can be passed with the XAPI_USER env variable.
    #[structopt(long = "user", short = "u")]
    user: Option<String>,
    /// XenServer host user password. Can be passed with the XAPI_PASSWORD env variable.
    #[structopt(long = "pass", short = "p")]
    pass: Option<String>,
    /// Output the result as non-prettified json
    #[structopt(long = "compact")]
    compact: bool,
    /// Case sensitive value for the xapi class
    class: String,
    /// Case sensitive value for the xapi method
    method: String,
    /// Ordered list of arguments for the call (if any). Do not pass a session.
    #[structopt(parse(from_str = "xapirpc::as_value_heuristic"))]
    args: Vec<xmlrpc::Value>,
}

main!(|cli_args: Cli| {
    let config = Config::new(
        &cli_args.host,
        &cli_args.user,
        &cli_args.pass,
        cli_args.compact,
    );

    let class = cli_args.class;
    let method = cli_args.method;
    let args = cli_args.args;

    if let Err(e) = xapirpc::run(&config, &class, &method, args) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
});
