Minimal CLI client for xapi rpc calls.
The output is in json, so it can be piped to `jq` or other json tools for further filtering.
For example you can get the `uuid` and `name_label` of all VMs with:
```bash
xapirpc VM get_all_records | jq '.[]|select(.is_a_template==false)|{uuid, name_label}'
```

There are a few flags available for customisation:
```bash
$ xapirpc --help
Minimal xapi xmlrpc CLI client
USAGE:
    xapirpc [FLAGS] [OPTIONS] <class> <method> [args]...
FLAGS:
        --compact    Output the result as non-prettified json
        --help       Prints help information
    -V, --version    Prints version information
OPTIONS:
    -h, --host <host>    XenServer host. Can be passed with the XAPI_HOST env variable.
    -p, --pass <pass>    XenServer host user password. Can be passed with the XAPI_PASSWORD env variable.
    -u, --user <user>    XenServer host user name. Can be passed with the XAPI_USER env variable.
ARGS:
    <class>      Case sensitive value for the xapi class
    <method>     Case sensitive value for the xapi method
    <args>...    Ordered list of arguments for the call (if any). Do not pass a session.
```

The `host`, `user`, and `pass` value can be manually configured by creating
```
$HOME/.config/xapirpc/config.prefs.json
```
and adding them in a json object. E.g.
```bash
$ cat $HOME/.config/xapirpc/config.prefs.json
{"user":"my_user_name","pass":"my_pass"}
```

To try it, clone this repository and build with `cargo build --release` or install it using `cargo install xapirpc --force`.

The crate exports also a `xapirpc` crate that exposes some potentially useful helpers to create a xapi client. The executable provides an example of use.
# Acknowledgements

Thanks:

- @gaborigloi for porting the library to `quicli`
- @Pistahh for fixing the `jq` example and comments on the code

