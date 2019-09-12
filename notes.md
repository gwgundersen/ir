`libc` appears to define `wait4()` in the `linux_like` submodule only, not BSD,
so I added it:

```
--- a/src/unix/bsd/mod.rs
+++ b/src/unix/bsd/mod.rs
@@ -690,6 +690,8 @@ extern {
                           f: extern fn(*mut ::c_void) -> *mut ::c_void,
                           value: *mut ::c_void) -> ::c_int;
     pub fn acct(filename: *const ::c_char) -> ::c_int;
+    pub fn wait4(pid: ::pid_t, status: *mut ::c_int, options: ::c_int,
+                 rusage: *mut ::rusage) -> ::pid_t;
 }
 
 cfg_if! {
```

To tell Cargo to use it:
```
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -7,3 +7,9 @@ edition = "2018"
 exitcode = "1.1.2"
 libc = "0.2"
 
+[patch.crates-io]
+libc = { path = "/Users/alex/src/libc" }
+
```

