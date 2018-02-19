Minimal CLI client for xapi rpc calls.
The ouptu is in json, so can be piped to `jq` or other json tools for further filtering.
For example you can get the `uuid` and `name__label` of all the VMs with:
```bash
xapirpc VM get_all_records | jq '.|to_entries|.[]|.value|select(.is_a_template==false)|{uuid,name_label}'
```

There are a few optional flags available to customise the usage:
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
