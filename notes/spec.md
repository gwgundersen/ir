
```js
{
    "argv": [
        "/bin/echo",
        "Hello, world!"
    ]
}
```

# Keys

- `argv` (list): Argument vector.

- `exe` (string): Path to executable.  If ommitted or null, uses `argv[0]`.

- `path` (list or string): Lookup path for executable, if it contains no
  slashes.  If `true`, uses the `PATH` environment variable in ir's own
  environment (not in the process environment).  If false, does not look up 
  the executable in a lookup path.

- `env` (object): How to construct the process environment.

    - `inherit` (bool or array): If true, inherit all the env vars.  If false,
      don't inherit any vars.  If an array, inherit named env vars, if any.

    - `vars` (object): Env vars to set or override.  If the value is null, the
      env var is removed if present.

- `cwd` (string): Current working directory.  If omitted or null, inherits.

- `stdin`, `stdout`, `stderr`:

    - `{"type": "inherit"}` or null or omitted: Inherit the fd.

    - `{"type": "close"}` or false: Close the fd.  Ignores failures.

    - `{"type": "file", "path": path}` or (string): File; may be relative to
      ir's CWD (not process CWD).  Also,
      
          - `mode`
          - `group`

    - `{"type": "fd", "fd": n}` or (int): `dup`ed to incoming file descriptor.

    - `{"type": "capture"}`: Capture in memory and emit in JSON.  Must be UTF-8?
    
    - `{"format": "raw"}`: Raw.
    
    - `{"format": "text"}`: Raw but cleaned for UTF-8. (??)

    - `{"format": "capture"}`: Timestamped binary transicript format.

    - FIXME: tcp, udp, http, websocket?
    
- `transicript` (object): Where to write transicribed fds, if any.  As for other
  fds, but "inherit", "close", "transicript" not allowed.  Also,
  
    - `max_len` (int): Maximum length to read in one chunk.  Default 1048576.

- `fd` (list)

- `umask` (int): Process umask, or null to inherit.

- `pdeath_sig` (int): Parent death signal, or null for none.

- FIXME: multiple procs in a single file, run in parallel
- FIXME: signal disposition
- FIXME: wait, or don't, or daemonize
- FIXME: report child pid to caller, somehow?
- FIXME: Poll for usage, other status, update intermediate file?
- FIXME: state file
- FIXME: state web service?
- FIXME: shell command?
- FIXME: YAML and other spec formats?


### Error handling

Choices:

0. Nothing.
1. Return exit code.
2. Write to some fd or a file a JSON obj with status.



### Examples

```json
{
    "stdout": "/path/to/stdout",
    "stderr": 1
}
```

```json
{
    "stdout": {"type": "file", "path": "/path/to/stdout"},
    "stderr": {"type": "fd", "fd": 1},
}
```

```json
{
    "stdout": {"file": "/path/to/stdout"},
    "stderr": {"fd": 1},
}
```

```json
{
    "stdout": {"file": {"path": "/path/to/stdout"}},
    "stderr": {"fd": {"fd": 1}},
}
```


# Transcript file format

- `u64` magic number: 46 FE 54 52 4E 53 00 00

  (Last two bytes are LE version number.)

- `timeval` base time.

- Repeat:

    - `u32` fd
    - `u32` length in bytes
    - `u64` nano offset from base time

