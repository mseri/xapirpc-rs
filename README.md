

```
Dummy xapi xmlrpc CLI client 0.1
Marcello S. <marcello.seri@citrix.com>
CLI interface to interrogate an instance of XenServer via xmlrpc
USAGE:
    xapirpc [OPTIONS] <CLASS> <METHOD> [ARGS]...
FLAGS:
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

To try it, clone this repository and build with `cargo build --release`
