use callback::CallbackSubscription;
use callback::SubscribableCallback;
use core::isize;
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

struct ReturnCode(isize);

impl From<isize> for ReturnCode {
    fn from(v: isize) -> ReturnCode {
        ReturnCode(v)
    }
}

impl From<ReturnCode> for Error {
    fn from(v: ReturnCode) -> Error {
        match v.0 {
            -1 => Error::Fail,         // Generic failure condition
            -2 => Error::Busy,         // Underlying system is busy; retry
            -3 => Error::Already,      // The state requested is already set
            -4 => Error::Off,          // The component is powered down
            -5 => Error::Reserve,      // Reservation required before use
            -6 => Error::Invalid,      // An invalid parameter was passed
            -7 => Error::Size,         // Parameter passed was too large
            -8 => Error::Cancel,       // Operation cancelled by a call
            -9 => Error::Nomem,        // Memory required not available
            -10 => Error::Nosupport,   // Operation or command is unsupported
            -11 => Error::Nodevice,    // Device does not exist
            -12 => Error::Uninstalled, //Device is not physically installed
            -13 => Error::Noack,       // Packet transmission not acknowledged
            _ => unreachable!(),
        }
    }
}

impl From<ReturnCode> for Result<u32, Error> {
    fn from(c: ReturnCode) -> Result<u32, Error> {
        match c.0 {
            v @ 0...isize::MAX => Ok(v as u32),
            e => Err(c.into()),
        }
    }
}

pub fn yieldk() {
    // Note: A process stops yielding when there is a callback ready to run,
    // which the kernel executes by modifying the stack frame pushed by the
    // hardware. The kernel copies the PC value from the stack frame to the LR
    // field, and sets the PC value to callback to run. When this frame is
    // unstacked during the interrupt return, the effectively clobbers the LR
    // register.
    //
    // At this point, the callback function is now executing, which may itself
    // clobber any of the other caller-saved registers. Thus we mark this
    // inline assembly as conservatively clobbering all caller-saved registers,
    // forcing yield to save any live registers.
    //
    // Upon direct observation of this function, the LR is the only register
    // that is live across the SVC invocation, however, if the yield call is
    // inlined, it is possible that the LR won't be live at all (commonly seen
    // for the `loop { yieldk(); }` idiom) or that other registers are live,
    // thus it is important to let the compiler do the work here.
    //
    // According to the AAPCS: A subroutine must preserve the contents of the
    // registers r4-r8, r10, r11 and SP (and r9 in PCS variants that designate
    // r9 as v6) As our compilation flags mark r9 as the PIC base register, it
    // does not need to be saved. Thus we must clobber r0-3, r12, and LR
    unsafe {
        asm!(
            "svc 0"
            :
            :
            : "memory", "r0", "r1", "r2", "r3", "r12", "lr"
            : "volatile");
    }
}

pub fn yieldk_for<F: Fn() -> bool>(cond: F) {
    while !cond() {
        yieldk();
    }
}

pub fn subscribe<CB: SubscribableCallback>(
    driver_number: usize,
    subscribe_number: usize,
    callback: &mut CB,
) -> Result<CallbackSubscription, Error> {
    let return_code = unsafe {
        subscribe_ptr(
            driver_number,
            subscribe_number,
            c_callback::<CB> as *const _,
            callback as *mut CB as usize,
        )
    };

    return_code.map(|_| CallbackSubscription::new(driver_number, subscribe_number))
}

extern "C" fn c_callback<CB: SubscribableCallback>(
    arg0: usize,
    arg1: usize,
    arg2: usize,
    userdata: usize,
) {
    let callback = unsafe { &mut *(userdata as *mut CB) };
    callback.call_rust(arg0, arg1, arg2);
}

pub unsafe fn subscribe_ptr(
    major: usize,
    minor: usize,
    cb: *const unsafe extern "C" fn(usize, usize, usize, usize),
    ud: usize,
) -> Result<u32, Error> {
    let res;
    asm!("svc 1" : "={r0}"(res)
                 : "{r0}"(major) "{r1}"(minor) "{r2}"(cb) "{r3}"(ud)
                 : "memory"
                 : "volatile");
    ReturnCode(res).into()
}

pub unsafe fn command(major: usize, minor: usize, arg1: usize, arg2: usize) -> Result<u32, Error> {
    let res;
    asm!("svc 2" : "={r0}"(res)
                 : "{r0}"(major) "{r1}"(minor) "{r2}"(arg1) "{r3}"(arg2)
                 : "memory"
                 : "volatile");
    ReturnCode(res).into()
}

pub fn allow(
    driver_number: usize,
    allow_number: usize,
    buffer_to_share: &mut [u8],
) -> Result<SharedMemory, Error> {
    let len = buffer_to_share.len();
    let return_code = unsafe {
        allow_ptr(
            driver_number,
            allow_number,
            buffer_to_share.as_mut_ptr(),
            len,
        )
    };

    return_code.map(|_| SharedMemory::new(driver_number, allow_number, buffer_to_share))
}

pub unsafe fn allow_ptr(
    major: usize,
    minor: usize,
    slice: *mut u8,
    len: usize,
) -> Result<u32, Error> {
    let res;
    asm!("svc 3" : "={r0}"(res)
                 : "{r0}"(major) "{r1}"(minor) "{r2}"(slice) "{r3}"(len)
                 : "memory"
                 : "volatile");
    ReturnCode(res).into()
}

pub unsafe fn memop(major: u32, arg1: usize) -> Result<u32, Error> {
    let res;
    asm!("svc 4" : "={r0}"(res)
                 : "{r0}"(major) "{r1}"(arg1)
                 : "memory"
                 : "volatile");
    ReturnCode(res).into()
}
