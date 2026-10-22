use core::cell::UnsafeCell;

#[repr(transparent)]
pub struct VolatileCell<T>(pub UnsafeCell<T>);

impl<T> VolatileCell<T> {
    pub fn read(&self) -> T {
        unsafe { core::ptr::read_volatile(self.0.get()) }
    }

    pub fn write(&self, value: T) {
        unsafe { core::ptr::write_volatile(self.0.get(), value) }
    }

    pub fn write_other<S>(&self, value: S) {
        unsafe { core::ptr::write_volatile(self.0.get() as *mut S, value) }
    }
}
