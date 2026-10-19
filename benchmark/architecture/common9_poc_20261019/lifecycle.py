"""One host-clock admission gate; no fixture/oracle semantics."""
import threading,time

class ObservationCancelled(RuntimeError):pass

class ObservationGate:
 def __init__(self,clock=time.monotonic_ns):
  self.lock=threading.RLock();self.clock=clock;self.deadline_ns=None;self.cancelled_ns=None;self.pending_update_cutoffs={}
 def check_locked(self):
  now=self.clock()
  cutoffs=[x for x in [self.deadline_ns,*self.pending_update_cutoffs.values()]if x is not None]
  if self.cancelled_ns is not None or cutoffs and now>=min(cutoffs):
   raise ObservationCancelled('OBSERVATION_DEADLINE_CANCELLED_NO_NEW_DISPATCH')
  return now
 def check(self):
  with self.lock:return self.check_locked()
 def update(self,deadline_ns):
  with self.lock:
   now=self.check_locked()
   if deadline_ns<=now:raise ObservationCancelled('DEADLINE_UPDATE_ALREADY_EXPIRED')
   if self.deadline_ns is not None and deadline_ns<self.deadline_ns:raise ValueError('Deadline cannot shorten')
   self.deadline_ns=deadline_ns
   return now
 def cancel(self):
  with self.lock:
   if self.cancelled_ns is None:self.cancelled_ns=self.clock()
   return self.cancelled_ns
