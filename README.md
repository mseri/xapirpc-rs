Minimal CLI client for xapi rpc calls.
The ouptu is in json, so can be piped to `jq` or other json tools for further filtering.
For example you can get the `uuid` and `name_label` of all the VMs with:
```bash
xapirpc VM get_all_records | jq '.[]|select(.is_a_template==false)|{uuid, name_label}'
```

There are a few optional flags available to customise the usage:
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

The `host`, `user` and `pass` value can be manually configured by creating
```
$HOME/.config/xapirpc/config.prefs.json
```
and adding them in a json object. E.g.
```bash
$ cat $HOME/.config/xapirpc/config.prefs.json
{"user":"my_user_name","pass":"my_pass"}
```

To try it, clone this repository and build with `cargo build --release` or install it using `cargo install xapirpc --force`.

# Acknowledgements

Thanks:

- @gaborigloi for porting the library to `quicli`
