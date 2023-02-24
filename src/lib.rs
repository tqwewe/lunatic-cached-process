//! **Cached process lookups with [lunatic](https://crates.io/crates/lunatic).**
//!
//! When a process is lookup, it is cached in the local process to avoid unnecessery future lookups.
//! This is useful for globally registered processes and abstract processes.
//!
//! # Example
//!
//! ```
//! use lunatic::{spawn_link, test};
//! use lunatic_cached_process::{cached_process, CachedLookup, ProcessCached};
//!
//! cached_process! {
//!     static COUNTER_PROCESS: ProcessCached<()> = "counter-process";
//! }
//!
//! let process = spawn_link!(|mailbox: Mailbox<()>| { loop { } });
//! process.register("counter-process");
//!
//! let lookup: Option<Process<T>> = COUNTER_PROCESS.get(); // First call lookup process from host
//! assert!(lookup.is_some());
//!
//! let lookup: Option<Process<T>> = COUNTER_PROCESS.get(); // Subsequent calls will use cached process
//! assert!(lookup.is_some());
//! ```

use std::cell::RefCell;

use lunatic::{process::ProcessRef, Process, ProcessLocal};
use serde::{Deserialize, Serialize};

pub type ProcessCached<T> = CachedProcess<Process<T>>;
pub type ProcessRefCached<T> = CachedProcess<ProcessRef<T>>;

/// Cached process to avoid looking up a global process multiple times.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CachedProcess<T> {
    // TODO: Replace with `Cell` when lunatic gets a new version where `ProcessRef` is `Copy`.
    lookup_state: RefCell<LookupState<T>>,
    process_name: &'static str,
}

impl<T> CachedProcess<T> {
    /// Construct a new process cache with a registered process name.
    pub fn new(name: &'static str) -> Self {
        CachedProcess {
            lookup_state: RefCell::new(LookupState::NotLookedUp),
            process_name: name,
        }
    }

    /// Sets the process cache value.
    pub fn set(&self, value: T) {
        *self.lookup_state.borrow_mut() = LookupState::Present(value);
    }

    /// Resets the process cache, causing subsequent calls to lookup the process again.
    pub fn reset(&self) {
        *self.lookup_state.borrow_mut() = LookupState::NotLookedUp;
    }
}

/// Trait for accessing a static process local cache.
pub trait CachedLookup<T> {
    /// Looks up a process by its name, and caches the result.
    /// Subsequent calls will used the cached value.
    fn get(self) -> Option<T>;

    /// Sets the cached lookup. This will prevent any lookups from being made,
    /// since subsequent calls to [`CachedLookup::get`] will return this cached value.
    fn set(self, value: T);

    /// Resets the cache, causing the next call to [`CachedLookup::get`] to lookup the process again.
    fn reset(self);
}

macro_rules! impl_cached_lookup {
    ($ty:ident, $cache_ty:ident) => {
        impl<T> CachedLookup<$ty<T>> for &'static ProcessLocal<$cache_ty<T>> {
            fn get(self) -> Option<$ty<T>> {
                lookup(self, |name| $ty::lookup(name))
            }

            fn set(self, value: $ty<T>) {
                set(self, value);
            }

            fn reset(self) {
                reset(self)
            }
        }
    };
}

impl_cached_lookup!(Process, ProcessCached);
impl_cached_lookup!(ProcessRef, ProcessRefCached);

/// Macro for defining a process local lookup cache for processes.
///
/// # Examples
///
/// Cached [`lunatic::Process`].
///
/// ```
/// use lunatic_cached_process::{cached_process, ProcessCached};
/// #
/// # enum CounterMessage {}
///
/// cached_process! {
///     static COUNTER: ProcessCached<CountMessage> = "global-counter-process";
/// }
/// ```
///
/// Cached [`lunatic::process::ProcessRef`].
///
/// ```
/// use lunatic_cached_process::{cached_process, ProcessRefCached};
/// #
/// # struct CounterProcess {}
/// # impl lunatic::process::AbstractProcess for CounterProcess {
/// #     type Arg = ();
/// #     type State = Self;
/// #     
/// # }
///
/// cached_process! {
///     static COUNTER: ProcessRefCached<CounterProcess> = "global-counter-process-ref";
/// }
/// ```
#[macro_export]
macro_rules! cached_process {
    (
        static $ident:ident : $ty:ty = $name:tt ;
    ) => {
        lunatic::process_local! {
            static $ident: $ty = $crate::CachedProcess::new($name);
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

fn lookup<F, T>(cached_process: &'static ProcessLocal<CachedProcess<T>>, f: F) -> Option<T>
where
    F: Fn(&'static str) -> Option<T>,
    T: Clone,
{
    cached_process.with(|proc| {
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
    })
}

fn set<T>(cached_process: &'static ProcessLocal<CachedProcess<T>>, value: T) {
    cached_process.with(|proc| proc.set(value))
}

fn reset<T>(cached_process: &'static ProcessLocal<CachedProcess<T>>) {
    cached_process.with(|proc| proc.reset())
}
