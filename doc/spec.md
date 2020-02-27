This page explains the syntax for a _spec_, which describes in detail how to run
processes.  The `procs` key is a list of specifications of processes to run.

```js
{
  "procs": [
    {...},
    {...}
  ]
}
```

If there is only one proc, the enclosing array may be omitted.


# Procs

Each process to run is given by an object.

```js
{
  "argv": [program, ...],
  "envs": {...},
  "fds": [...]
}
```


### Argv

An array of strings givign the argument vector list.  (required)

The first element is used also used as the executable name.


### Envs

How to construct the process environment.  (optional)

```js
{
  "inherit": ...,
  "vars": {
    "name": "value",
    ...
  }
}
```

The `inherit` key may be:
- `true`, to inherit all env vars from the parent process
- `false`, to inherit no env vars from the parent process
- an array of env var names to inherit

The `vars` key is an object whose keys and values are used as environment
variables.  Values must be strings.  These take precedence over inherited env
vars of the same names.


### Fds

How to set up file descriptors for the process.  (optional)

```js
{
  "fds": [
    [fd, fd_spec],
    [fd, fd_spec],
    ...
  ]
}
```

An array of pairs; each pair gives a file descriptor and a specification for how
to set it up.

`fd` is a string containing a nonnegative integer file descriptor, or one of the
following aliases:
- `"stdin"` for 0
- `"stdout"` for 1
- `"stderr"` for 2

`fd_spec` may be:

- `"inherit"`: The file descriptor is inherited from the parent, if it is open
  in the parent.  This is the default behavior for all file descriptors.
  
- `"close"`: The file descriptor is closed, if it is open in the parent.

- `{"null": {"flags": ...}}`: The file descriptor is opened to `/dev/null`.  The
  open flags may be specified (see below).  Such a file descriptor is different
  from a closed file descriptor: whereas a write to a closed file descriptor
  will fail, a write to a `/dev/null` file descriptor will succeed but the data
  discarded.

- 
    ```js
    {
      "file": {
        "path": path,
        "flags": open_flags,        # optional
        "mode": file_mode,          # optional
      }
    }
    ```
  The file descriptor is opened to the named path.

- `{"dup": {"fd": fd}}`: The file descriptor is duplicated from another file
  descriptor `fd`.  The process's file descriptor setup rules are applied in the
  order given, so `fd` may refer to a previously set up file descriptor.

-
    ```js
    {
      "capture": {
        "mode": capture_mode,       # optional
        "format" capture_format,    # optional
      }
    }
    ```
  The output of the file descriptor is captured from the running process, and
  included in the process results.
    
  `capture_mode` may be: 

  - `"tempfile"` (default): Open the file descriptor to an unlinked temporary
    file, which receives the output.  When the process terminates, the contents
    of the file are loaded into the result; the file descriptor is closed and
    the temporary file deleted.
    
  - `"memory"`: Read from the file descriptor into a buffer in ir's own memory,
    via a pipe.  This means ir's memory usage will grow as the process produces
    more output.
    
  `capture_format` specifies how to represent the captured data, and may be:
  
  - `"text"` (default): Treat the data as UTF-8-encded text, and include it in
    the results as a string.  If the data contains invalid UTF-8, it is
    sanitized to be valid text; this is a lossy operation.
    
  - `"base64"`: Encode data as base64.

