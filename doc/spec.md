This page explains the syntax for a _spec_, which describes in detail how to run
processes.

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
  open flags may be specified (see below).

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

  - `"tempfile"`: Open the file descriptor to an unlinked temporary file, which
    receives the output.  When the process terminates, the contents of the file
    are loaded into the result; the file descriptor is closed and the temporary
    file deleted.
    
  `capture_format` specifies how to represent the captured data, and may be:
  
  - `"text"`: Treat the data as UTF-8-encded text, and include it in the results
    as a string.  If the data contains invalid UTF-8, it is sanitized to be valid
    text.
    
