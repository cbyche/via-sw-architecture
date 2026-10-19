//! Technical host-monotonic observation lease. No semantic or fixture knowledge.
use std::sync::{Arc, Mutex, MutexGuard, atomic::{AtomicU64,Ordering}};

#[cfg(target_os = "macos")]
pub fn host_monotonic_ns() -> u64 {
    #[repr(C)]
    struct Timebase { numer: u32, denom: u32 }
    unsafe extern "C" {
        fn mach_absolute_time() -> u64;
        fn mach_timebase_info(info: *mut Timebase) -> i32;
    }
    let mut base = Timebase { numer: 0, denom: 0 };
    unsafe {
        assert_eq!(mach_timebase_info(&mut base), 0, "host clock timebase unavailable");
        assert!(base.denom > 0);
        ((mach_absolute_time() as u128 * base.numer as u128) / base.denom as u128) as u64
    }
}
#[cfg(not(target_os = "macos"))]
pub fn host_monotonic_ns() -> u64 { panic!("Common9 host-clock adapter requires macOS") }

#[derive(Default)]
pub struct EffectState {
    pub deadline_ns: Option<u64>,
    cancel_signal: Arc<AtomicU64>,
}
impl EffectState {
    pub fn check_at(&self, now: u64) -> Result<(), String> {
        if self.cancel_signal.load(Ordering::SeqCst)!=0 || self.deadline_ns.is_some_and(|deadline| now >= deadline) {
            Err("OBSERVATION_DEADLINE_CANCELLED_NO_STATE_EFFECT".into())
        } else { Ok(()) }
    }
}
#[derive(Clone)]
pub struct EffectGate {
    state: Arc<Mutex<EffectState>>,
    clock: Arc<dyn Fn() -> u64 + Send + Sync>,
    cancel_signal: Arc<AtomicU64>,
}
impl Default for EffectGate {
    fn default() -> Self {
        Self::with_clock(Arc::new(host_monotonic_ns))
    }
}
impl EffectGate {
    fn with_clock(clock: Arc<dyn Fn()->u64+Send+Sync>) -> Self {
        let state=EffectState::default();let cancel_signal=state.cancel_signal.clone();
        Self{state:Arc::new(Mutex::new(state)),clock,cancel_signal}
    }
    pub fn guard(&self) -> Result<MutexGuard<'_, EffectState>, String> {
        let state = self.state.lock().unwrap();
        let now = (self.clock)();
        state.check_at(now)?;
        Ok(state)
    }
    pub fn update(&self, deadline_ns: u64) -> Result<u64, String> {
        let mut state = self.guard()?;
        let at=(self.clock)();state.check_at(at)?;
        if deadline_ns <= at { return Err("DEADLINE_UPDATE_ALREADY_EXPIRED".into()); }
        if state.deadline_ns.is_some_and(|old| deadline_ns < old) {
            return Err("DEADLINE_UPDATE_CANNOT_SHORTEN".into());
        }
        state.deadline_ns = Some(deadline_ns);
        Ok(at)
    }
    pub fn cancel(&self) -> u64 {
        let now = (self.clock)();
        // Signal before any blocking state/stdout lock. Already acquired guards
        // recheck this atomic signal immediately before their bounded mutation.
        match self.cancel_signal.compare_exchange(0,now,Ordering::SeqCst,Ordering::SeqCst){Ok(_)=>now,Err(previous)=>previous}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    #[test]
    fn exact_cutoff_blocks_commit_and_cannot_be_revived() {
        let now = Arc::new(AtomicU64::new(10)); let c = now.clone();
        let gate = EffectGate::with_clock(Arc::new(move || c.load(Ordering::SeqCst)));
        gate.update(20).unwrap(); now.store(19, Ordering::SeqCst); assert!(gate.guard().is_ok());
        now.store(20, Ordering::SeqCst); assert!(gate.guard().is_err()); assert!(gate.update(40).is_err());
        assert_eq!(gate.cancel(), 20); now.store(21, Ordering::SeqCst); assert_eq!(gate.cancel(), 20);
    }
    #[test]
    fn live_late_source_can_extend_only_before_old_cutoff() {
        let now = Arc::new(AtomicU64::new(10)); let c = now.clone();
        let gate = EffectGate::with_clock(Arc::new(move || c.load(Ordering::SeqCst)));
        gate.update(20).unwrap(); now.store(19, Ordering::SeqCst); gate.update(30).unwrap();
        now.store(29, Ordering::SeqCst); assert!(gate.guard().is_ok()); gate.cancel(); assert!(gate.guard().is_err());
    }
}
