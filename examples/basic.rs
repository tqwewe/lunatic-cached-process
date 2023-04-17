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
