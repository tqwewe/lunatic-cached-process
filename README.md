**Cached process lookups with [lunatic](https://crates.io/crates/lunatic).**

When a process is lookup, it is cached in the local process to avoid unnecessery future lookups.
This is useful for globally registered processes and abstract processes.

# Example

```rust
use lunatic::{spawn_link, Process};
use lunatic_cached_process::{cached_process, CachedLookup};

cached_process! {
    static COUNTER_PROCESS: Process<()> = "counter-process";
}

fn main() {
    let process = spawn_link!(|_mailbox: Mailbox<()>| { loop {} });
    process.register("counter-process");

    let lookup: Option<Process<()>> = COUNTER_PROCESS.get(); // First call lookup process from host
    assert!(lookup.is_some());

    let lookup: Option<Process<()>> = COUNTER_PROCESS.get(); // Subsequent calls will use cached process
    assert!(lookup.is_some());
}
```

For more examples, see the [examples] directory.

[examples]: https://github.com/tqwewe/lunatic-cached-process/tree/main/examples

## License

Licensed under either

- [Apache License 2.0]
- [MIT License]

at your option.

[Apache License 2.0]: ./LICENSE-APACHE
[MIT License]: ./LICENSE-MIT
