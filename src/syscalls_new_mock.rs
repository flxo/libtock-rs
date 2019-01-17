use callback::CallbackSubscription;
use callback::SubscribableCallback;
use shared_memory::SharedMemory;

pub enum Error {
    Fail,           // Generic failure condition
    Busy,           // Underlying system is busy; retry
    Already,        // The state requested is already set
    Off,            // The component is powered down
    Reserve,        // Reservation required before use
    Invalid,        // An invalid parameter was passed
    Size,           // Parameter passed was too large
    Cancel,         // Operation cancelled by a call
    Nomem,          // Memory required not available
    Nosupport,      // Operation or command is unsupported
    Nodevice,       //Device does not exist
    Uninstalled,    // Device is not physically installed
    Noack,          // Packet transmission not acknowledged
    Unknown(isize), // Unknown Error
}

pub fn yieldk() {
    unimplemented!()
}

pub fn yieldk_for<F: Fn() -> bool>(cond: F) {
    unimplemented!()
}

pub fn subscribe<CB: SubscribableCallback>(
    driver_number: usize,
    subscribe_number: usize,
    callback: &mut CB,
) -> Result<CallbackSubscription, Error> {
    unimplemented!()
}

extern "C" fn c_callback<CB: SubscribableCallback>(
    arg0: usize,
    arg1: usize,
    arg2: usize,
    userdata: usize,
) {
    unimplemented!()
}

pub unsafe fn subscribe_ptr(
    major: usize,
    minor: usize,
    cb: *const unsafe extern "C" fn(usize, usize, usize, usize),
    ud: usize,
) -> Result<u32, Error> {
    unimplemented!()
}

pub unsafe fn command(major: usize, minor: usize, arg1: usize, arg2: usize) -> Result<u32, Error> {
    unimplemented!()
}

pub fn allow(
    driver_number: usize,
    allow_number: usize,
    buffer_to_share: &mut [u8],
) -> Result<SharedMemory, Error> {
    unimplemented!()
}

pub unsafe fn allow_ptr(
    major: usize,
    minor: usize,
    slice: *mut u8,
    len: usize,
) -> Result<u32, Error> {
    unimplemented!()
}

pub unsafe fn memop(major: u32, arg1: usize) -> Result<u32, Error> {
    unimplemented!()
}
