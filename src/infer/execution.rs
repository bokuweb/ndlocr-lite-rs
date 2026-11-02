//! Native scoped workers retain their scheduling; wasm executes each job inline.
#[cfg(not(target_arch = "wasm32"))]
pub use std::thread::{ScopedJoinHandle, scope};

#[cfg(target_arch = "wasm32")]
pub struct Scope;
#[cfg(target_arch = "wasm32")]
pub struct ScopedJoinHandle<'a, T>(T, std::marker::PhantomData<&'a ()>);
#[cfg(target_arch = "wasm32")]
pub fn scope<T>(f: impl FnOnce(&Scope) -> T) -> T {
    f(&Scope)
}
#[cfg(target_arch = "wasm32")]
impl Scope {
    pub fn spawn<'a, T>(&'a self, f: impl FnOnce() -> T) -> ScopedJoinHandle<'a, T> {
        ScopedJoinHandle(f(), std::marker::PhantomData)
    }
}
#[cfg(target_arch = "wasm32")]
impl<T> ScopedJoinHandle<'_, T> {
    pub fn join(self) -> std::thread::Result<T> {
        Ok(self.0)
    }
}
