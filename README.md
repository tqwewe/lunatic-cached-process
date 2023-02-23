**Cached process lookups with [lunatic](https://crates.io/crates/lunatic).**

When a process is lookup, it is cached in the local process to avoid unnecessery future lookups.
This is useful for globally registered processes and abstract processes.

# Example

```rust
use lunatic::{spawn_link, test};
use lunatic_cached_process::{cached_process, CachedLookup, ProcessCached};

cached_process! {
    static COUNTER_PROCESS: ProcessCached<()> = "counter-process";
}

let process = spawn_link!(|mailbox: Mailbox<()>| { loop { } });
process.register("counter-process");

let lookup: Option<Process<T>> = COUNTER_PROCESS.get(); // First call lookup process from host
assert!(lookup.is_some());

let lookup: Option<Process<T>> = COUNTER_PROCESS.get(); // Subsequent calls will use cached process
assert!(lookup.is_some());
```

## License

Licensed under either

- [Apache License 2.0]
- [MIT License]

at your option.

[Apache License 2.0]: ./LICENSE-APACHE
[MIT License]: ./LICENSE-MIT
