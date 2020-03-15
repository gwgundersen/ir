- [ ] handle signals and shut down cleanly
- [ ] forward signals to subprocess
- [ ] pdeath_sig
- [ ] signal disposition
- [ ] record start time, end time, elapsed time
- [ ] serialize timeval as float sec instead of {tv_sec, tv_usec}
- [ ] seriealize ru_maxrss in bytes
- [ ] clean up oflags
- [ ] accept a map for fds, if you don't care about order
- [ ] exe
- [ ] cwd
- [ ] cwd before interpreting spec?
- [ ] umask
- [ ] results to file, via --output option or similar
- [ ] 'stdin', 'stdout', 'stderr' aliases to fds, consistently
- spec validation
  - [ ] no fd is given more than once
- [ ] feed input into fd
- [ ] fd to named temporary file, with path in result
- [ ] periodic update of results file while running
- [ ] rusage for self vs children
- [ ] input fd (stdin etc) from file
- [ ] when running multiple procs, a way to connect their fds via pipes
- [ ] transcript
- [ ] transcript client lib (Python?)
- [ ] don't wait; fire and forget (certain options only)
- [ ] daemonize
- [ ] report child pid to caller, somehow?
- [ ] poll for usage, other status, update intermediate file?
- [ ] state file
- [ ] state web service?
- [ ] shell command?
- [ ] YAML and other spec formats?
- [ ] process groups???
- [ ] build a spec from a running process
- [ ] compression support for output files
- [ ] accept {fd: spec,...} instead of [[fd, spec],...], if order is unimportant
- [ ] input/output fd from/to network (tcp, udp, websocket, REST API)
- file opening improvements
  - [ ] specify file mode as "0600"
  - [ ] specify file mode as "rw-r-----"
  - [ ] specify file mode as "u+rw g+r"
  - [ ] special file mode, that overrides umask
  - [ ] specify group for created file
  - [ ] create parent dirs for created file

### Maybe

- [snafu](https://docs.rs/snafu/0.5.0/snafu/guide/index.html) for error types?


### Done

- [x] split out lib
- [x] set up integration test
- [x] default "flags": "Default" in fd spec
- [x] output fd (stdout, stderr, etc) to file
- [x] open() mode on O_CREAT
- [x] add exit code / signum to result JSON
- [x] merge output fds
- [x] results fds as associative list
- [x] capture fd into results via tempfile
- [x] set up user docs
- [x] spec error type
- [x] add file to fd results
- [x] rename Result -> Res
- [x] create fd::Error and fd::Result
- [x] capture fd into results via pipe
- [x] capture to bytes, encode base64 in JSON
- [x] top level union error type
- [x] integration tests for capture
- [x] integration test for UTF8 sanitization
- [x] error reporting model, for parent process
- [x] capture child proc setup errors and send back to parent
- [x] multiple procs in a single file, run in parallel

