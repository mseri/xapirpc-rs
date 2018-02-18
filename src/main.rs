extern crate base64;

#[macro_use]
extern crate clap;

extern crate iso8601;
extern crate preferences;
extern crate reqwest;
extern crate serde_json;
extern crate xmlrpc;

use std::str::FromStr;

use base64::encode;
use clap::{Arg, App};
use preferences::{AppInfo, Preferences, PreferencesMap};
use reqwest::Client;
use serde_json::value as json;
use xmlrpc::{Request, Value};


fn heuristic_to_value(value: &str) -> Value {
    if let Ok(b) = bool::from_str(value) {
        return Value::Bool(b);
    }

    if let Ok(i) = i64::from_str(value) {
        return Value::Int64(i);
    }

    if let Ok(f) = f64::from_str(value) {
        return Value::Double(f);
    }

    return Value::String(value.to_string());
}

// From xmlrpc-rs' utils.rs
pub fn format_datetime(date_time: &iso8601::DateTime) -> String {
    let iso8601::Time { hour, minute, second, .. } = date_time.time;

    match date_time.date {
        iso8601::Date::YMD { year, month, day } => {
            format!("{:04}{:02}{:02}T{:02}:{:02}:{:02}",
                year, month, day,
                hour, minute, second
            )
        }
        _ => { unimplemented!() }
    }
}

trait Helpers {
    fn get_value(&self) -> &xmlrpc::Value;
    fn extract_session(self) -> String;
    fn as_json(&self) -> json::Value;
}

impl Helpers for xmlrpc::Value {
    // The two functions below should be changed to return a Result
    // instead of panicking
    fn get_value(&self) -> &Value {
        if let Value::Struct(ref response) = *self {
            &response["Value"]
        } else {
            panic!("Malformed response: {:?}", self)
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

fn main() {
    let matches = App::new("Minimal xapi xmlrpc CLI client")
        .about("CLI interface to interrogate an instance of XenServer via xmlrpc")
        .version(crate_version!())
        .arg(Arg::with_name("host")
             .long("host")
             .value_name("HOST")
             .env("XAPI_HOST")
             .help("XenServer host. Can be passed with the HOST env variable.")
             .takes_value(true))
        .arg(Arg::with_name("user")
             .short("u")
             .long("user")
             .value_name("USER")
             .env("XAPI_USER")
             .help("XenServer host user name. Can be passed with the \
                   XAPI_USER env variable.")
             .takes_value(true))
        .arg(Arg::with_name("pass")
             .short("p")
             .long("pass")
             .value_name("PASSWORD")
             .env("XAPI_PASSWORD")
             .help("XenServer host user password. Can be passed with the \
                   XAPI_PASSWORD env variable.")
             .takes_value(true))
        .arg(Arg::with_name("compact")
             .long("compact")
             .help("Output the result as non-prettified json.")
            )
        .arg(Arg::with_name("class")
             .value_name("CLASS")
             .help("Case sensitive value for the xapi class.")
             .required(true)
             .index(1))
        .arg(Arg::with_name("method")
             .value_name("METHOD")
             .help("Case sensitive value for the xapi method.")
             .required(true)
             .index(2))
        .arg(Arg::with_name("args")
             .value_name("ARGS")
             .help("Ordered list of arguments for the call (if any). \
                   Do not pass a session.")
             .multiple(true))
        .get_matches();

    let host_default = "http://127.0.0.1".to_string();
    let user_default = "guest".to_string();
    let pass_default = "guest".to_string();

    let preferences = PreferencesMap::<String>::load(&APP_INFO, "config").unwrap_or(
        PreferencesMap::<String>::new()
        //config.insert("user".into(), "guest".into());
        //config.insert("pass".into(), "guest".into());
        //config.inster("host".into(), "http://127.0.0.1".into());
        );

    let host = matches.value_of("host").unwrap_or(
        preferences.get("host").unwrap_or(&host_default));
    let user = matches.value_of("user").unwrap_or(
        preferences.get("user").unwrap_or(&user_default));
    let pass = matches.value_of("pass").unwrap_or(
        preferences.get("pass").unwrap_or(&pass_default));

    // These are compulsory parameters. unwrapping here is fine
    let class = matches.value_of("class").unwrap();
    let method = matches.value_of("method").unwrap();

    let args = if matches.is_present("args") {
        matches.values_of("args").unwrap().collect()
    } else {
        Vec::new()
    }.into_iter()
        .map(|a| heuristic_to_value(&a));

    let client = Client::new();

    // Let's panic all the way!!!
    let hopefully_session =
        Request::new("session.login_with_password")
        .arg(user).arg(pass)
        .call(&client, host)
        .unwrap()  // Response
        .unwrap(); // Result
    let session = hopefully_session.extract_session();
    //println!("Session: \"{}\"", session);

    // Prepare the xmlrpc command
    let cmd = format!("{}.{}", class, method);
    let mut req =
        Request::new(&cmd)
        .arg(session.clone());
    for arg in args {
        req = req.arg(arg);
    }

    // Again, we unwrap the Response and then the Result
    // don't care about the panics right now.
    let response = req.call(&client, host).unwrap().unwrap();

    let json_value = response.get_value().as_json();
    let j = if matches.is_present("compact") {
        serde_json::to_string(&json_value)
    } else {
        serde_json::to_string_pretty(&json_value)
    };
    println!("{}", j.unwrap());

    let _ =
        Request::new("session.logout")
        .arg(session)
        .call(&client, host);
}
