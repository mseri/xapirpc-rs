extern crate base64;

#[macro_use]
extern crate quicli;

extern crate iso8601;
extern crate preferences;
extern crate reqwest;
extern crate serde_json;
extern crate xmlrpc;

use std::env;
use std::error::Error;
use std::str::FromStr;

use base64::encode;
use preferences::{AppInfo, Preferences, PreferencesMap};
use quicli::prelude::*;
use reqwest::{Client, ClientBuilder};
use serde_json::value as json;
use xmlrpc::{Request, Value};

const APP_INFO: AppInfo = AppInfo {
    name: "xapirpc",
    author: "xapirpc",
};

fn as_value_heuristic(value: &str) -> Value {
    if let Ok(b) = bool::from_str(value) {
        return Value::Bool(b);
    }

    if let Ok(i) = i64::from_str(value) {
        return Value::Int64(i);
    }

    if let Ok(f) = f64::from_str(value) {
        return Value::Double(f);
    }

    Value::String(value.to_string())
}

fn as_url(host: &str) -> String {
    let prefix = if !(host.starts_with("https:") || host.starts_with("http:")) {
        "https://"
    } else {
        ""
    };

    format!("{}{}", prefix, host)
}

// From xmlrpc-rs' utils.rs
fn format_datetime(date_time: &iso8601::DateTime) -> String {
    let iso8601::Time {
        hour,
        minute,
        second,
        ..
    } = date_time.time;

    match date_time.date {
        iso8601::Date::YMD { year, month, day } => format!(
            "{:04}{:02}{:02}T{:02}:{:02}:{:02}",
            year, month, day, hour, minute, second
        ),
        _ => unimplemented!(),
    }
}

fn get(req: &Request, client: &Client, host: &str) -> Value {
    match req.call(client, host) {
        Ok(Ok(val)) => val,
        Ok(Err(e)) => panic!("Unexpected xmlrpc error: {:?}", e),
        Err(err) => panic!("{}", err.description()),
    }
}

trait Helpers {
    fn get_value(&self) -> &xmlrpc::Value;
    fn extract_session(self) -> String;
    fn as_json(&self) -> json::Value;
}

impl Helpers for xmlrpc::Value {
    fn get_value(&self) -> &Value {
        match *self {
            Value::Struct(ref response) if response.contains_key("Value") => &response["Value"],
            Value::Struct(ref response) if response.contains_key("ErrorDescription") => panic!(
                "XML Rpc error: {}",
                serde_json::to_string(&response["ErrorDescription"].as_json()).unwrap()
            ),
            _ => panic!("Unkown error: {:?}", self),
        }
    }

    fn extract_session(self) -> String {
        let value = self.get_value();
        if let Value::String(ref session) = *value {
            session.clone()
        } else {
            panic!("Mismatched type: {:?}", value)
        }
    }

    fn as_json(&self) -> json::Value {
        match *self {
            Value::Int(i) => {
                let i = json::Number::from_f64(i as f64).unwrap();
                json::Value::Number(i)
            }
            Value::Int64(i) => {
                let i = json::Number::from_f64(i as f64).unwrap();
                json::Value::Number(i)
            }
            Value::Bool(b) => json::Value::Bool(b),
            Value::String(ref s) => json::Value::String(s.clone()),
            Value::Double(d) => {
                let d = json::Number::from_f64(d).unwrap();
                json::Value::Number(d)
            }
            Value::DateTime(date_time) => json::Value::String(format_datetime(&date_time)),
            Value::Base64(ref data) => json::Value::String(encode(data)),
            Value::Struct(ref map) => {
                let mut jmap = serde_json::Map::with_capacity(map.len());
                for (ref name, ref v) in map {
                    jmap.insert(name.to_string().clone(), v.as_json());
                }
                json::Value::Object(jmap)
            }
            Value::Array(ref array) => {
                json::Value::Array(array.iter().map(|v| v.as_json()).collect())
            }
            Value::Nil => json::Value::Null,
        }
    }
}

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
    /// Disable ssl certificate verification
    #[structopt(long = "no-ssl")]
    disable_ssl_check: bool,
    /// Case sensitive value for the xapi class
    class: String,
    /// Case sensitive value for the xapi method
    method: String,
    /// Ordered list of arguments for the call (if any). Do not pass a session.
    #[structopt(parse(from_str = "as_value_heuristic"))]
    args: Vec<Value>,
}

main!(|cli_args: Cli| {
    let preferences = PreferencesMap::<String>::load(&APP_INFO, "config")
        .unwrap_or(PreferencesMap::<String>::new());

    let host_default = "http://127.0.0.1".to_string();
    let user_default = "guest".to_string();
    let pass_default = "guest".to_string();

    let host_env = env::var("XAPI_HOST").ok();
    let host = as_url(
        cli_args
            .host
            .as_ref()
            .or(host_env.as_ref())
            .or(preferences.get("host"))
            .unwrap_or(&host_default),
    );
    let user_env = env::var("XAPI_USER").ok();
    let user = cli_args
        .user
        .as_ref()
        .or(user_env.as_ref())
        .or(preferences.get("user"))
        .unwrap_or(&user_default);
    let pass_env = env::var("XAPI_PASSWORD").ok();
    let pass = cli_args
        .pass
        .as_ref()
        .or(pass_env.as_ref())
        .or(preferences.get("pass"))
        .unwrap_or(&pass_default);

    let class = cli_args.class;
    let method = cli_args.method;
    let args = cli_args.args;

    let client = {
        let mut client = ClientBuilder::new();
        if cli_args.disable_ssl_check {
            client.danger_disable_certificate_verification_entirely();
        }
        client.build()?
    };

    // Get the session
    let req = Request::new("session.login_with_password")
        .arg(user.as_str())
        .arg(pass.as_str()); // Result
    let hopefully_session = get(&req, &client, &host);
    let session = hopefully_session.extract_session();

    // Prepare the xmlrpc command
    let cmd = format!("{}.{}", class, method);
    let mut req = Request::new(&cmd).arg(session.clone());
    for arg in args {
        req = req.arg(arg);
    }

    let response = get(&req, &client, &host);

    let json_value = response.get_value().as_json();
    let j = if cli_args.compact {
        serde_json::to_string(&json_value)
    } else {
        serde_json::to_string_pretty(&json_value)
    };
    println!("{}", j.unwrap());

    // Logout to release the session
    let _ = Request::new("session.logout")
        .arg(session)
        .call(&client, &host);
});
