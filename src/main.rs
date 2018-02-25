extern crate preferences;

#[macro_use]
extern crate quicli;

extern crate serde_json;
extern crate xapirpc;
extern crate xmlrpc;

use std::process;

use preferences::{AppInfo, Preferences, PreferencesMap};
use quicli::prelude::*;

use xapirpc::{Config, XapiRpc};

const HOST: &'static str = "http://127.0.0.1";
const USER: &'static str = "guest";
const PASS: &'static str = "guest";

const APP_INFO: AppInfo = AppInfo {
    name: "xapirpc",
    author: "xapirpc",
};

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

struct Env {
    host: Option<String>,
    user: Option<String>,
    pass: Option<String>,
}

main!(|cli_args: Cli| {
    let preferences = PreferencesMap::<String>::load(&APP_INFO, "config")
        .unwrap_or(PreferencesMap::<String>::new());

    let env = Env {
        host: std::env::var("XAPI_HOST").ok(),
        user: std::env::var("XAPI_USER").ok(),
        pass: std::env::var("XAPI_PASSWORD").ok(),
    };

    let config = get_config(&cli_args, &env, &preferences);

    let compact = cli_args.compact;

    let class = cli_args.class;
    let method = cli_args.method;
    let args = cli_args.args;

    let xapi_session = XapiRpc::new(&config).unwrap_or_else(|e| {
        eprintln!("Error preparing the xapi session: {}", e);
        process::exit(1);
    });

    let response = xapi_session
        .call(&class, &method, args)
        .and_then(|v| {
            if compact {
                serde_json::to_string(&v)
            } else {
                serde_json::to_string_pretty(&v)
            }.map_err(|err| From::from(err))
        })
        .unwrap_or_else(|e| {
            eprintln!("Error: {}", e);
            process::exit(1);
        });

    println!("{}", response);
});

/// Get Config cascading the value selection through Cli, Env, and local Preferences
fn get_config<'a>(cli_args: &'a Cli, env: &'a Env, preferences: &'a PreferencesMap) -> Config {
    let host = as_url(
        cli_args
            .host
            .as_ref()
            .or(env.host.as_ref())
            .or(preferences.get("host"))
            .unwrap_or(&HOST.to_string()),
    );

    let user = cli_args
        .user
        .as_ref()
        .or(env.user.as_ref())
        .or(preferences.get("user"))
        .unwrap_or(&USER.to_string())
        .clone();

    let pass = cli_args
        .pass
        .as_ref()
        .or(env.pass.as_ref())
        .or(preferences.get("pass"))
        .unwrap_or(&PASS.to_string())
        .clone();

    Config { host, user, pass }
}

/// Backward compatible, kind of, host to url
fn as_url(host: &str) -> String {
    let prefix = if !(host.starts_with("https:") || host.starts_with("http:")) {
        "https://"
    } else {
        ""
    };

    format!("{}{}", prefix, host)
}
