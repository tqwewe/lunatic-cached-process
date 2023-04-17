//! **Cached process lookups with [lunatic](https://crates.io/crates/lunatic).**
//!
//! When a process is lookup, it is cached in the local process to avoid unnecessery future lookups.
//! This is useful for globally registered processes and abstract processes.
//!
//! # Example
//!
//! ```
//! use lunatic::{spawn_link, test};
//! use lunatic_cached_process::{cached_process, CachedLookup};
//!
//! cached_process! {
//!     static COUNTER_PROCESS: Process<()> = "counter-process";
//! }
//!
//! let process = spawn_link!(|mailbox: Mailbox<()>| { loop { } });
//! process.register("counter-process");
//!
//! let lookup: Option<Process<()>> = COUNTER_PROCESS.get(); // First call will lookup process from lunatic runtime
//! assert!(lookup.is_some());
//!
//! let lookup: Option<Process<()>> = COUNTER_PROCESS.get(); // Subsequent calls will use cached lookup
//! assert!(lookup.is_some());
//! ```

use std::cell::RefCell;

use lunatic::{ap::ProcessRef, serializer::Bincode, AbstractProcess, Process, ProcessLocal};
use serde::{Deserialize, Serialize};

/// This is used internally for the cached_process! macro.
#[doc(hidden)]
pub use paste::paste;

pub type ProcessCached<'a, T, S = Bincode> = CachedProcess<'a, Process<T, S>>;
pub type ProcessRefCached<'a, T> = CachedProcess<'a, ProcessRef<T>>;

/// Cached process to avoid looking up a global process multiple times.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CachedProcess<'a, T> {
    // TODO: Replace with `Cell` when lunatic gets a new version where `ProcessRef` is `Copy`.
    lookup_state: RefCell<LookupState<T>>,
    process_name: &'a str,
}

impl<'a, T> CachedProcess<'a, T> {
    /// Construct a new process cache with a registered process name.
    pub fn new(name: &'a str) -> Self {
        CachedProcess {
            lookup_state: RefCell::new(LookupState::NotLookedUp),
            process_name: name,
        }
    }

    /// Returns the process name.
    pub fn process_name(&'a self) -> &'a str {
        self.process_name
    }

    /// Returns true if the process has been looked up and exists.
    ///
    /// # Example
    ///
    /// ```
    /// use lunatic::Process;
    ///
    /// let process: CachedProcess<'static, Process<()>> = CachedProcess::new("foo");
    /// assert!(!process.is_present()); // Initially not present
    ///
    /// process.get();
    /// assert!(!process.is_present()); // Not present, even after lookup
    ///
    /// spawn!(|| { loop { /* ... */ } }).register("foo"); // Start a process called "foo"
    ///
    /// process.reset();
    /// process.get();
    /// assert!(process.is_present()); // Is present
    /// ```
    pub fn is_present(&'a self) -> bool {
        matches!(&*self.lookup_state.borrow(), LookupState::Present(_))
    }

    /// Returns true if the process has been looked up, regardless if the process was found.
    ///
    /// # Example
    ///
    /// ```
    /// use lunatic::Process;
    ///
    /// let process: CachedProcess<'static, Process<()>> = CachedProcess::new("");
    /// assert!(!process.is_looked_up());
    ///
    /// process.get();
    /// assert!(process.is_looked_up());
    /// ```
    pub fn is_looked_up(&'a self) -> bool {
        matches!(&*self.lookup_state.borrow(), LookupState::NotLookedUp)
    }
}

/// Trait for accessing a static process local cache.
pub trait CachedLookup<'a, T> {
    /// Looks up a process by its name, and caches the result.
    /// Subsequent calls will used the cached value.
    fn get(&'a self) -> Option<T>;

    /// Sets the cached lookup. This will prevent any lookups from being made,
    /// since subsequent calls to [`CachedLookup::get`] will return this cached value.
    fn set(&'a self, value: T);

    /// Resets the cache, causing the next call to [`CachedLookup::get`] to lookup the process again.
    fn reset(&'a self);
}

impl<T, S> CachedLookup<'static, Process<T, S>> for ProcessLocal<ProcessCached<'_, T, S>> {
    #[inline]
    fn get(&'static self) -> Option<Process<T, S>> {
        self.with(|proc| lookup(proc, |name| Process::lookup(name)))
    }

    #[inline]
    fn set(&'static self, value: Process<T, S>) {
        self.with(|proc| CachedLookup::set(proc, value))
    }

    #[inline]
    fn reset(&'static self) {
        self.with(CachedLookup::reset)
    }
}

impl<T, S> CachedLookup<'static, Process<T, S>> for ProcessCached<'_, T, S> {
    #[inline]
    fn get(&'static self) -> Option<Process<T, S>> {
        lookup(self, |name| Process::lookup(name))
    }

    #[inline]
    fn set(&'static self, value: Process<T, S>) {
        *self.lookup_state.borrow_mut() = LookupState::Present(value);
    }

    #[inline]
    fn reset(&'static self) {
        *self.lookup_state.borrow_mut() = LookupState::NotLookedUp;
    }
}

impl<T> CachedLookup<'static, ProcessRef<T>> for ProcessLocal<ProcessRefCached<'_, T>>
where
    T: AbstractProcess,
{
    #[inline]
    fn get(&'static self) -> Option<ProcessRef<T>> {
        self.with(|proc| lookup(proc, |name| ProcessRef::lookup(name)))
    }

    #[inline]
    fn set(&'static self, value: ProcessRef<T>) {
        self.with(|proc| CachedLookup::set(proc, value))
    }

    #[inline]
    fn reset(&'static self) {
        self.with(CachedLookup::reset)
    }
}

impl<T> CachedLookup<'static, ProcessRef<T>> for ProcessRefCached<'_, T>
where
    T: AbstractProcess,
{
    #[inline]
    fn get(&'static self) -> Option<ProcessRef<T>> {
        lookup(self, |name| ProcessRef::lookup(name))
    }

    #[inline]
    fn set(&'static self, value: ProcessRef<T>) {
        *self.lookup_state.borrow_mut() = LookupState::Present(value);
    }

    #[inline]
    fn reset(&'static self) {
        *self.lookup_state.borrow_mut() = LookupState::NotLookedUp;
    }
}

/// Macro for defining a process local lookup cache for processes.
///
/// The structure is as follows:
///
/// ```
/// static <ident>: <process_type> = <process_name>;
/// ```
///
/// Where
///
/// - `<ident>`: Static variable name.
/// - `<process_type>`: Either `Process<T>`, `ProcessRef<T>`, or `Process<T, S>` where `T` is the message type, and `S` is the serializer.
/// - `<process_name>`: The string literal of the process name.
///
/// # Examples
///
/// Cached [`lunatic::Process`].
///
/// ```
/// use lunatic_cached_process::cached_process;
/// use serde::{Serialize, Deserialize};
///
/// cached_process! {
///     static COUNTER: Process<CountMessage> = "global-counter-process";
/// }
///
/// #[derive(Serialize, Deserialize)]
/// enum CountMessage {
///     Inc,
///     Dec,
/// }
/// ```
///
/// Cached [`lunatic::process::ProcessRef`].
///
/// ```
/// use lunatic::{
///     ap::{AbstractProcess, Config, ProcessRef},
///     serializer::Bincode,
/// };
/// use lunatic_cached_process::cached_process;
///
/// cached_process! {
///     static COUNTER: ProcessRef<CounterProcess> = "global-counter-process-ref";
/// }
///
/// struct Counter(i32);
///
/// impl AbstractProcess for Counter {
///     type State = Self;
///     type Serializer = Bincode;
///     type Arg = i32;
///     type Handlers = ();
///     type StartupError = ();
///
///     fn init(
///         _config: Config<Self>,
///         initial_count: Self::Arg,
///     ) -> Result<Self::State, Self::StartupError> {
///         Ok(Counter(initial_count))
///     }
/// }
/// ```
#[macro_export]
macro_rules! cached_process {
    (
        $(
            $(#[$attr:meta])* $vis:vis static $ident:ident : $process_type:ident <$ty:ty $( , $s:ty )?> = $name:tt ;
        )+
    ) => {
        $crate::paste! {
            $(
                lunatic::process_local! {
                    $(#[$attr])* $vis static $ident: $crate:: [<$process_type Cached>] <'static, $ty $( , $s )?> = $crate::CachedProcess::new($name);
                }
            )+
        }
    };
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
enum LookupState<T> {
    NotLookedUp,
    NotPresent,
    Present(T),
}

impl<T> Default for LookupState<T> {
    fn default() -> Self {
        LookupState::NotLookedUp
    }
}

#[inline]
fn lookup<'a, F, T>(proc: &'a CachedProcess<T>, f: F) -> Option<T>
where
    F: Fn(&'a str) -> Option<T>,
    T: Clone,
{
    let proc_ref = proc.lookup_state.borrow();
    match &*proc_ref {
        LookupState::NotLookedUp => {
            std::mem::drop(proc_ref);
            match f(proc.process_name) {
                Some(process) => {
                    *proc.lookup_state.borrow_mut() = LookupState::Present(process.clone()); // TODO: Replace clone with copy
                    Some(process)
                }
                None => {
                    *proc.lookup_state.borrow_mut() = LookupState::NotPresent;
                    None
                }
            }
        }
        LookupState::NotPresent => None,
        LookupState::Present(process) => {
            Some(process.clone()) // TODO: Replace clone with copy
        }
    }
}
