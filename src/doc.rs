use std::fmt::Display;

pub fn help(prefix: impl Into<String> + Display) -> String {
    format!(
        "## Help
**Ban**

Bans a user permanently from the server
```text
{prefix} ban USERID MESSAGE?
```

**Unban**

Bans a user permanently from the server
```text
{prefix} !ban USERID
```

**Kick**

Kicks a user from the server
```text
{prefix} kick USERID
```

**Delete**

Deletes a message by Id, (also works with replying)
```text
{prefix} delete MESSAGEID
```

**Warn**

Warns a user, with an optional reason
```text
{prefix} warn USERID MESSAGE?
```

**Warns**

Display count for warnings for user
```text
{prefix} warns USERID
```"
    )
}

pub fn links() -> String {
    "### Useful links
[Source](https://github.com/toastxc/autoguard)
[Issues](https://github.com/toastxc/autoguard/issues)
[Wiki](https://github.com/toastxc/autoguard/wiki)
"
    .to_string()
}
