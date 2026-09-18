use std::sync::{Mutex, MutexGuard};

/// Lock a mutex, recovering from poisoning instead of panicking.
///
/// A poisoned mutex means another thread panicked while holding the lock; the
/// protected data is still structurally valid for our use, so we recover it.
pub fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_lock_recovers_from_poison() {
        let m = Arc::new(Mutex::new(0i32));
        let m2 = m.clone();
        let _ = std::thread::spawn(move || {
            let _guard = lock(&m2);
            panic!("poison the mutex");
        })
        .join();

        // Mutex is now poisoned; our helper still returns a usable guard.
        let mut guard = lock(&m);
        *guard += 1;
        assert_eq!(*guard, 1);
    }
}
