extern crate base64;

extern crate iso8601;
extern crate reqwest;
extern crate serde_json;

#[macro_use]
extern crate simple_error;

extern crate xmlrpc;

use std::error::Error;
use std::str::FromStr;

use base64::encode;
use reqwest::Client;
use serde_json::value as json;
use xmlrpc::{Request, Value};

pub struct Config {
    pub host: String,
    pub user: String,
    pub pass: String,
    pub compact: bool,
}

pub fn as_value_heuristic(value: &str) -> Value {
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

fn get(req: &Request, client: &Client, host: &str) -> Result<xmlrpc::Value, Box<Error>> {
    match req.call(client, host) {
        Ok(Ok(val)) => Ok(val),
        Ok(Err(e)) => Err(Box::new(e)),
        Err(err) => Err(Box::new(err)),
    }
}

pub trait Helpers {
    fn get_value(&self) -> Result<&xmlrpc::Value, Box<Error>>;
    fn extract_session(self) -> Result<String, Box<Error>>;
    fn as_json(&self) -> json::Value;
}

impl Helpers for xmlrpc::Value {
    fn get_value(&self) -> Result<&Value, Box<Error>> {
        match *self {
            Value::Struct(ref response) if response.contains_key("Value") => Ok(&response["Value"]),
            Value::Struct(ref response) if response.contains_key("ErrorDescription") => {
                bail!(format!(
                    "XML Rpc error: {}",
                    serde_json::to_string(&response["ErrorDescription"].as_json())?
                ))
            }
            _ => bail!(format!("Unkown error: {:?}", self)),
        }
    }

    fn extract_session(self) -> Result<String, Box<Error>> {
        let value = self.get_value()?;
        if let Value::String(ref session) = *value {
            Ok(session.clone())
        } else {
            bail!(format!("Mismatched type: {:?}", value))
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

pub fn run(config: &Config, class: &str, method: &str, args: Vec<Value>) -> Result<(), Box<Error>> {
    let host = &config.host;
    let client = Client::new();

    // Get the session
    let req = Request::new("session.login_with_password")
        .arg(config.user.clone())
        .arg(config.pass.clone());
    let session = get(&req, &client, &host)?.extract_session()?;

    // Prepare the xmlrpc command
    let cmd = format!("{}.{}", class, method);
    let mut req = Request::new(&cmd).arg(session.clone());
    for arg in args {
        req = req.arg(arg)
    }

    let response = get(&req, &client, &host)?;

    let json_value = response.get_value()?.as_json();
    let j = if config.compact {
        serde_json::to_string(&json_value)?
    } else {
        serde_json::to_string_pretty(&json_value)?
    };
    println!("{}", j);

    // Logout to release the session
    let _ = Request::new("session.logout")
        .arg(session)
        .call(&client, &host);

    Ok(())
}
