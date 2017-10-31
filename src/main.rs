extern crate clap;
extern crate reqwest;
extern crate xmlrpc;

use std::str::FromStr;

use clap::{Arg, App};
use xmlrpc::{Request, Value};
use reqwest::Client;

fn to_value(value: &str) -> Value {
    bool::from_str(value)
        .map(|b| Value::Bool(b))
        .unwrap_or(
            i64::from_str(value)
            .map(|i| Value::Int64(i))
            .unwrap_or(
                f64::from_str(value)
                .map(|f| Value::Double(f))
                .unwrap_or(
                    Value::String(value.to_string())
                    )))
}


fn extract_session(res: Value) -> String {
    if let Value::Struct(response) = res {
        response.get("Value")
            .map(|val|
                 {
                     if let Value::String(ref session) = *val {
                         session.clone()
                     } else {
                         panic!("Mismatched type: {:?}", val)
                     }
                 })
        .unwrap()
    } else {
        panic!("Error: {:?}", res)
    }
}


fn main() {

    let matches = App::new("Dummy xapi xmlrpc CLI client")
        .version("0.1")
        .author("Marcello S. <marcello.seri@citrix.com>")
        .about("CLI interface to interrogate an instance of XenServer via xmlrpc")
        .arg(Arg::with_name("host")
             .long("host")
             .value_name("HOST")
             .env("XAPI_HOST")
             .help("XenServer host")
             .takes_value(true))
        .arg(Arg::with_name("user")
             .short("u")
             .long("user")
             .value_name("USER")
             .env("XAPI_USER")
             .help("XenServer host user name")
             .takes_value(true))
        .arg(Arg::with_name("pass")
             .short("p")
             .long("pass")
             .value_name("PASSWORD")
             .env("XAPI_PASSWORD")
             .help("XenServer host user password")
             .takes_value(true))
        .arg(Arg::with_name("class")
             .value_name("CLASS")
             .help("Case sensitive value for the xapi class")
             .required(true)
             .index(1))
        .arg(Arg::with_name("method")
             .value_name("METHOD")
             .help("Case sensitive value for the xapi method")
             .required(true)
             .index(2))
        .arg(Arg::with_name("args")
             .value_name("ARGS")
             .help("Ordered list of arguments for the call (if any)")
             .multiple(true))
        .get_matches();

    let host = matches.value_of("host").unwrap_or("http://127.0.0.1");
    let user = matches.value_of("user").unwrap_or("root");
    let pass = matches.value_of("pass").unwrap_or("xenroot");

    let class = matches.value_of("class").unwrap();
    let method = matches.value_of("method").unwrap();

    let args =
        if matches.is_present("args") {
            matches.values_of("args").unwrap().collect()
        } else {
            Vec::new()
        }
    .into_iter()
        .map(|a| to_value(&a));

    let client = Client::new().unwrap();

    // Let's panic all the way!!!
    let hopefully_session = Request::new("session.login_with_password").arg(user).arg(pass)
        .call(&client, host)
        .unwrap() // Response
        .unwrap(); // Result
    let session = extract_session(hopefully_session);
    println!("Session: \"{}\"", session);

    let cmd = format!("{}.{}", class, method);
    let mut req = Request::new(&cmd).arg(session.clone());
    for arg in args {
        req = req.arg(arg);
    }
    let response = req.call(&client, host).unwrap().unwrap();
    println!("Response: {:?}", response);

    let close = Request::new("session.logout").arg(session).call(&client, host);
    println!("Closed? {:?}", close);
}
