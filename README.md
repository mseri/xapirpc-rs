Minimal CLI client for xapi rpc calls.
The ouptu is in json, so can be piped to `jq` or other json tools for further filtering.

```
xapirpc --help
Minimal xapi xmlrpc CLI client
CLI interface to interrogate an instance of XenServer via xmlrpc

USAGE:
    xapirpc [FLAGS] [OPTIONS] <CLASS> <METHOD> [ARGS]...

FLAGS:
        --compact    Output the result as non-prettified json.
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --host <HOST>        XenServer host. Can be passed with the HOST env variable. [env:XAPI_HOST: ]
    -p, --pass <PASSWORD>    XenServer host user password. Can be passed with the XAPI_PASSWORD env variable.
                             [env:XAPI_PASSWORD: ]
    -u, --user <USER>        XenServer host user name. Can be passed with the XAPI_USER env variable. [env:XAPI_USER: ]

ARGS:
    <CLASS>      Case sensitive value for the xapi class.
    <METHOD>     Case sensitive value for the xapi method.
    <ARGS>...    Ordered list of arguments for the call (if any). Do not pass a session.

```

To try it, clone this repository and build with `cargo build --release` or install it using `cargo install xapirpc --force`.
