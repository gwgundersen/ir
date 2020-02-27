A program for running other programs


# Description

`ir` takes a [JSON specification](doc/spec.md) of processes to run, and runs
them.  The JSON specification gives detailed control over all aspects of the
process:

- the executable to run
- the command line
- the environment
- stdin [planned], stdout, stderr
- additional file descriptors
- current working directory [planned]
- umask [planned]
- signal dispositions [planned]
- parent death signal [planned]

`ir` runs the processes in concurrent subprocesses, collects detailed results,
and returns them in a JSON document.  Results include,

- process ID
- exit status
- resource usage
- file descriptor outputs, if requested


# Implementation

`ir` is a project to help me learn Rust.  I deliberately avoid many dependency
crates (other than the excellend Serde), to get experience with Rust systems
programming.

Integration tests are written in Python and run under `pytest`, because this is
easy to do.


