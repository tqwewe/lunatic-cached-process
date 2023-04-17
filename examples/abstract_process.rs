use lunatic::{
    ap::{AbstractProcess, Config, ProcessRef},
    serializer::Bincode,
};
use lunatic_cached_process::{cached_process, CachedLookup};

cached_process! {
    static COUNTER_ABSTRACT_PROCESS: ProcessRef<Counter> = "counter-abstract-process";
}

struct Counter(i32);

impl AbstractProcess for Counter {
    type State = Self;
    type Serializer = Bincode;
    type Arg = i32;
    type Handlers = ();
    type StartupError = ();

    fn init(
        _config: Config<Self>,
        initial_count: Self::Arg,
    ) -> Result<Self::State, Self::StartupError> {
        Ok(Counter(initial_count))
    }
}

fn main() {
    Counter::start_as("counter-abstract-process", 0).unwrap();

    let lookup: Option<ProcessRef<Counter>> = COUNTER_ABSTRACT_PROCESS.get(); // First call lookup process from host
    assert!(lookup.is_some());

    let lookup: Option<ProcessRef<Counter>> = COUNTER_ABSTRACT_PROCESS.get(); // Subsequent calls will use cached process
    assert!(lookup.is_some());
}
