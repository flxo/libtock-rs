use callback::{Returncode, CallbackSubscription, SubscribableCallback};
use result;
use syscalls;

pub const DRIVER_NUM: usize = 0x0005;
pub const BUFFER_SIZE: usize = 128;

mod command {
    pub const COUNT: usize = 0;
    pub const START: usize = 1;
    pub const START_REPEAT: usize = 2;
    pub const START_REPEAT_BUFFER: usize = 3;
    pub const START_REPEAT_BUFFER_ALT: usize = 4;
    pub const STOP: usize = 5;
}

mod subscribe {
    pub const SUBSCRIBE_CALLBACK: usize = 0;
}

mod allow {
    pub const BUFFER: usize = 0;
    pub const BUFFER_ALT: usize = 1;
}

#[derive(Debug)]
pub enum Error {
    NotSupported,
    SubscriptionFailed,
    Busy,
    Invalid,
    Fail,
    Other(ReturnCode),
}

impl<T: Sized> From<ReturnCode> for Result<T, Error> {
    fn from(code: RetunCode) -> Result<T, Error> {
        match code {
            Success(_) => Ok(()),
            Fail => Err(Error::Fail),
            Busy => Err(Error::Busy),
            Inval => Err(Error::Invalid),
            e => Err(Error::Other(e)),
        }
    }
}

#[repr(align(32))]
pub struct AdcBuffer {
    // TODO: make this generic if possible with the driver impl
    pub buffer: [u8; BUFFER_SIZE],
}

pub struct Adc<'a> {
    count: usize,
    #[allow(dead_code)]
    subscription: CallbackSubscription<'a>,
}

pub fn with_callback<CB>(callback: CB) -> WithCallback<CB> {
    WithCallback { callback }
}

pub struct WithCallback<CB> {
    callback: CB,
}

impl<CB: FnMut(usize, usize)> SubscribableCallback for WithCallback<CB> {
    fn call_rust(&mut self, _: usize, channel: usize, value: usize) {
        (self.callback)(channel, value);
    }
}

impl<'a, CB> WithCallback<CB>
where
    Self: SubscribableCallback,
{
    pub fn init(&mut self) -> Result<Adc, Error> {
        let count = unsafe { syscalls::command(DRIVER_NUM, command::COUNT, 0, 0) };
        if count < 1 {
            return Err(AdcError::NotSupported);
        }

        match syscalls::subscribe(DRIVER_NUM, subscribe::SUBSCRIBE_CALLBACK, self) {
            Ok(subscription) => {
                Adc {
                    count: count as usize,
                    subscription,
                }
            }
            Err(e) => e.into(),
        }
    }
}

impl<'a> Adc<'a> {
    pub fn init_buffer(
        self,
        buffer: &'a mut AdcBuffer,
        alt: Option<&'a mut AdcBuffer>,
    ) -> Result<Adc<'a>, AdcError> {
        syscalls::allow(DRIVER_NUM, allow::BUFFER, &mut buffer.buffer)
            .map(|_| ())
            .map_err(AdcError::Other)?; // TODO
        if let Some(alt) = alt {
            syscalls::allow(DRIVER_NUM, allow::BUFFER_ALT, &mut alt.buffer)
                .map(|_| ())
                .map_err(AdcError::Other)?; // TODO
        }
        Ok(self)
    }

    /// Return the number of available channels
    pub fn count(&self) -> usize {
        self.count
    }

    /// Start a single sample of channel
    pub fn sample(&self, channel: usize) -> Result<(), AdcError> {
        let return_code = unsafe { syscalls::command(DRIVER_NUM, command::START, channel, 0) };
        match return_code.into() {
            syscalls::ReturnCode::Success(_) => Ok(())
            _ => 
        }
            result::SUCCESS => Ok(()),
            result::EBUSY => Err(AdcError::Busy),
            result::FAIL => Err(AdcError::Fail),
            unexpected => Err(AdcError::Other(unexpected)),
        }
    }

    /// Start continonous sampling of channel
    pub fn sample_continous(&self, channel: usize) -> Result<(), AdcError> {
        unimplemented!()
    }

    /// Start continus sampling to first buffer
    pub fn sample_continous_buffered(&self) -> Result<(), AdcBuffer> {
        unimplemented!()
    }

    /// Start continus sampling to alternating buffer
    pub fn sample_continous_buffered_alt(&self) {
        unimplemented!()
    }

    /// Stop any started samplling operation
    pub fn stop(&self) {
        unsafe { syscalls::command(DRIVER_NUM, command::STOP, 0, 0) };
    }
}

impl<'a> Drop for Adc<'a> {
    fn drop(&mut self) {
        self.stop();
    }
}
