//! Requested Rust allocator bytes, not RSS or allocator metadata/retained pages.
use std::alloc::{GlobalAlloc,Layout,System};
use std::sync::atomic::{AtomicUsize,Ordering};
pub struct Tracked;
static LIVE:AtomicUsize=AtomicUsize::new(0);
static PEAK:AtomicUsize=AtomicUsize::new(0);
fn add(n:usize){let now=LIVE.fetch_add(n,Ordering::SeqCst)+n;PEAK.fetch_max(now,Ordering::SeqCst);}
unsafe impl GlobalAlloc for Tracked{
    unsafe fn alloc(&self,l:Layout)->*mut u8{let p=System.alloc(l);if !p.is_null(){add(l.size())}p}
    unsafe fn alloc_zeroed(&self,l:Layout)->*mut u8{let p=System.alloc_zeroed(l);if !p.is_null(){add(l.size())}p}
    unsafe fn dealloc(&self,p:*mut u8,l:Layout){System.dealloc(p,l);LIVE.fetch_sub(l.size(),Ordering::SeqCst);}
    unsafe fn realloc(&self,p:*mut u8,l:Layout,n:usize)->*mut u8{let q=System.realloc(p,l,n);if !q.is_null(){if n>=l.size(){add(n-l.size())}else{LIVE.fetch_sub(l.size()-n,Ordering::SeqCst);}}q}
}
pub fn live()->usize{LIVE.load(Ordering::SeqCst)}
pub fn peak()->usize{PEAK.load(Ordering::SeqCst)}
