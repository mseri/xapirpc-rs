extern crate base64;

extern crate iso8601;
extern crate reqwest;
extern crate serde_json;

#[macro_use]
extern crate simple_error;

extern crate xmlrpc;

use std::error::Error;
use std::str::FromStr;
use std::collections::BTreeMap;

use base64::encode;
use reqwest::Client;
use serde_json::value as json;
use xmlrpc::Request;

type XapiResult<T> = Result<T, Box<Error + Send + Sync>>;

/// Xapi RPC configuration.
pub struct Config {
    pub host: String,
    pub user: String,
    pub pass: String,
}

/// Xapi RPC client. Makes sure of creating, holding and closing the sessions.
pub struct XapiRpc {
    host: String,
    session: String,
    client: Client,
}

impl XapiRpc {
    /// Prepare a xapi session using the login informations from config.
    pub fn new(config: &Config) -> XapiResult<Self> {
        let host = config.host.clone();
        let client = Client::new();

        // Get the session
        let req = Request::new("session.login_with_password")
            .arg(config.user.clone())
            .arg(config.pass.clone());
        let session = get(&req, &client, &host)?.xapi_session()?;

        Ok(XapiRpc {
            host,
            session,
            client,
        })
    }

    /// Perform a Xapi RPC call for class.method using args as arguments
    pub fn call(
        &self,
        class: &str,
        method: &str,
        args: Vec<xmlrpc::Value>,
    ) -> XapiResult<json::Value> {
        let cmd = format!("{}.{}", class, method);
        let mut req = Request::new(&cmd).arg(self.session.clone());
        for arg in args {
            req = req.arg(arg)
        }

        let response = get(&req, &self.client, &self.host)?.rpc_value()?.as_json();

        Ok(response)
    }
}

// use Drop to close the session on exit
impl Drop for XapiRpc {
    fn drop(&mut self) {
        let _ = Request::new("session.logout")
            .arg(self.session.clone())
            .call(&self.client, &self.host);
    }
}

/// Helper to automatically convert strings to xmlrpc values trying to infer their types.
pub fn as_value_heuristic(value: &str) -> xmlrpc::Value {
    if let Ok(b) = bool::from_str(value) {
        return xmlrpc::Value::Bool(b);
    }

    if let Ok(i) = i64::from_str(value) {
        return xmlrpc::Value::Int64(i);
    }

    if let Ok(f) = f64::from_str(value) {
        return xmlrpc::Value::Double(f);
    }

    xmlrpc::Value::String(value.to_string())
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

fn get(req: &Request, client: &Client, host: &str) -> XapiResult<xmlrpc::Value> {
    match req.call(client, host) {
        Ok(Ok(val)) => Ok(val),
        Ok(Err(e)) => Err(Box::new(e)),
        Err(err) => Err(Box::new(err)),
    }
}

/// Utilities to extract RPC response value and convert them to serde json values
pub trait RpcHelpers {
    /// Translate the json value into an equivalent xmlrpc value.
    /// This is not 1-1, as we have no reasonable way to detect
    /// base64, date or i32 values from json.
    fn from_json(value: &json::Value) -> xmlrpc::Value;
    /// Convert the RPC value to a serde json value
    fn as_json(&self) -> json::Value;
    /// Get the "Value" field from a RPC response
    fn rpc_value(&self) -> XapiResult<&xmlrpc::Value>;
}

impl RpcHelpers for xmlrpc::Value {
    fn from_json(value: &json::Value) -> xmlrpc::Value {
        match *value {
            json::Value::Number(ref i) if i.is_i64() => {
                let n = i.as_i64().unwrap();
                xmlrpc::Value::Int64(n)
            }
            // not an i64, we make it into f64
            json::Value::Number(ref i) => {
                let n = i.as_f64().unwrap();
                xmlrpc::Value::Double(n)
            }
            json::Value::Bool(b) => xmlrpc::Value::Bool(b),
            json::Value::String(ref s) => xmlrpc::Value::String(s.clone()),
            json::Value::Object(ref jmap) => {
                let mut map = BTreeMap::new();
                for (ref name, ref v) in jmap {
                    map.insert(name.to_string().clone(), Self::from_json(v));
                }
                xmlrpc::Value::Struct(map)
            }
            json::Value::Array(ref array) => {
                xmlrpc::Value::Array(array.iter().map(|v| Self::from_json(v)).collect())
            }
            json::Value::Null => xmlrpc::Value::Nil,
        }
    }

    fn as_json(&self) -> json::Value {
        match *self {
            xmlrpc::Value::Int(i) => {
                let i = json::Number::from_f64(i as f64).unwrap();
                json::Value::Number(i)
            }
            xmlrpc::Value::Int64(i) => {
                let i = json::Number::from_f64(i as f64).unwrap();
                json::Value::Number(i)
            }
            xmlrpc::Value::Bool(b) => json::Value::Bool(b),
            xmlrpc::Value::String(ref s) => json::Value::String(s.clone()),
            xmlrpc::Value::Double(d) => {
                let d = json::Number::from_f64(d).unwrap();
                json::Value::Number(d)
            }
            xmlrpc::Value::DateTime(date_time) => json::Value::String(format_datetime(&date_time)),
            xmlrpc::Value::Base64(ref data) => json::Value::String(encode(data)),
            xmlrpc::Value::Struct(ref map) => {
                let mut jmap = serde_json::Map::with_capacity(map.len());
                for (ref name, ref v) in map {
                    jmap.insert(name.to_string().clone(), v.as_json());
                }
                json::Value::Object(jmap)
            }
            xmlrpc::Value::Array(ref array) => {
                json::Value::Array(array.iter().map(|v| v.as_json()).collect())
            }
            xmlrpc::Value::Nil => json::Value::Null,
        }
    }

    fn rpc_value(&self) -> XapiResult<&xmlrpc::Value> {
        match *self {
            xmlrpc::Value::Struct(ref response) if response.contains_key("Value") => {
                Ok(&response["Value"])
            }
            xmlrpc::Value::Struct(ref response) if response.contains_key("ErrorDescription") => {
                bail!(format!(
                    "XML Rpc error: {}",
                    serde_json::to_string(&response["ErrorDescription"].as_json())?
                ))
            }
            _ => bail!(format!("Unkown error: {:?}", self)),
        }
    }
}

trait SessionHelper {
    fn xapi_session(self) -> XapiResult<String>;
}

impl SessionHelper for xmlrpc::Value {
    /// Extract the xapi session from a XML RPC response
    fn xapi_session(self) -> XapiResult<String> {
        let value = self.rpc_value()?;
        if let xmlrpc::Value::String(ref session) = *value {
            Ok(session.clone())
        } else {
            bail!(format!("Mismatched type: {:?}", value))
        }
    }
}
