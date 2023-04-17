use lunatic::process::{AbstractProcess, ProcessRef, StartProcess};
use lunatic_cached_process::{cached_process, CachedLookup};

cached_process! {
    static COUNTER_ABSTRACT_PROCESS: ProcessRef<Counter> = "counter-abstract-process";
}

struct Counter(i32);

impl AbstractProcess for Counter {
    type Arg = i32;
    type State = Self;

    fn init(_this: ProcessRef<Self>, initial_count: Self::Arg) -> Self::State {
        Counter(initial_count)
    }
}

fn main() {
    Counter::start(0, Some("counter-abstract-process"));

    let lookup: Option<ProcessRef<Counter>> = COUNTER_ABSTRACT_PROCESS.get(); // First call lookup process from host
    assert!(lookup.is_some());

    let lookup: Option<ProcessRef<Counter>> = COUNTER_ABSTRACT_PROCESS.get(); // Subsequent calls will use cached process
    assert!(lookup.is_some());
}
