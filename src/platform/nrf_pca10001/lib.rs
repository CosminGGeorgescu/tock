#![crate_name = "platform"]
#![crate_type = "rlib"]
#![no_std]
#![feature(lang_items)]

extern crate drivers;
extern crate hil;
extern crate nrf51822;
extern crate support;
extern crate process;

use hil::Controller;
use drivers::timer::AlarmToTimer;
use drivers::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};

pub struct Firestorm {
    chip: nrf51822::chip::Nrf51822,
    gpio: &'static drivers::gpio::GPIO<'static, nrf51822::gpio::GPIOPin>,
    timer: &'static drivers::timer::TimerDriver<'static, AlarmToTimer<'static,
                                VirtualMuxAlarm<'static, nrf51822::rtc::Rtc>>>,
}

impl Firestorm {
    pub unsafe fn service_pending_interrupts(&mut self) {
        self.chip.service_pending_interrupts()
    }

    pub unsafe fn has_pending_interrupts(&mut self) -> bool {
        self.chip.has_pending_interrupts()
    }

    #[inline(never)]
    pub fn with_driver<F, R>(&mut self, driver_num: usize, f: F) -> R where
            F: FnOnce(Option<&hil::Driver>) -> R {
        match driver_num {
            1 => f(Some(self.gpio)),
            3 => f(Some(self.timer)),
            _ => f(None)
        }
    }
}

macro_rules! static_init {
   ($V:ident : $T:ty = $e:expr) => {
        let $V : &mut $T = {
            // Waiting out for size_of to be available at compile-time to avoid
            // hardcoding an abitrary large size...
            static mut BUF : [u8; 1024] = [0; 1024];
            let mut tmp : &mut $T = mem::transmute(&mut BUF);
            *tmp = $e;
            tmp
        };
   }
}

pub unsafe fn init<'a>() -> &'a mut Firestorm {
    use core::mem;
    use nrf51822::gpio::PA;

    static mut FIRESTORM_BUF : [u8; 1024] = [0; 1024];

    //XXX: this should be pared down to only give externally usable pins to the
    //  user gpio driver
    static_init!(gpio_pins : [&'static nrf51822::gpio::GPIOPin; 32] = [
            &nrf51822::gpio::PA[ 0],
            &nrf51822::gpio::PA[ 1],
            &nrf51822::gpio::PA[ 2],
            &nrf51822::gpio::PA[ 3],
            &nrf51822::gpio::PA[ 4],
            &nrf51822::gpio::PA[ 5],
            &nrf51822::gpio::PA[ 6],
            &nrf51822::gpio::PA[ 7],
            &nrf51822::gpio::PA[ 8],
            &nrf51822::gpio::PA[ 9],
            &nrf51822::gpio::PA[10],
            &nrf51822::gpio::PA[11],
            &nrf51822::gpio::PA[12],
            &nrf51822::gpio::PA[13],
            &nrf51822::gpio::PA[14],
            &nrf51822::gpio::PA[15],
            &nrf51822::gpio::PA[16],
            &nrf51822::gpio::PA[17],
            &nrf51822::gpio::PA[18],
            &nrf51822::gpio::PA[19],
            &nrf51822::gpio::PA[20],
            &nrf51822::gpio::PA[21],
            &nrf51822::gpio::PA[22],
            &nrf51822::gpio::PA[23],
            &nrf51822::gpio::PA[24],
            &nrf51822::gpio::PA[25],
            &nrf51822::gpio::PA[26],
            &nrf51822::gpio::PA[27],
            &nrf51822::gpio::PA[28],
            &nrf51822::gpio::PA[29],
            &nrf51822::gpio::PA[30],
            &nrf51822::gpio::PA[31],
            ]);
    static_init!(gpio : drivers::gpio::GPIO<'static, nrf51822::gpio::GPIOPin> =
                 drivers::gpio::GPIO::new(gpio_pins));
    for pin in gpio_pins.iter() {
        pin.set_client(gpio);
    }

    let rtc = &nrf51822::rtc::RTC;

    static_init!(mux_alarm : MuxAlarm<'static, nrf51822::rtc::Rtc> =
                    MuxAlarm::new(&nrf51822::rtc::RTC));
    rtc.configure(mux_alarm);

    static_init!(virtual_alarm1 : VirtualMuxAlarm<'static, nrf51822::rtc::Rtc> =
                    VirtualMuxAlarm::new(mux_alarm));
    static_init!(vtimer1 : AlarmToTimer<'static,
                                VirtualMuxAlarm<'static, nrf51822::rtc::Rtc>> =
                            AlarmToTimer::new(virtual_alarm1));
    virtual_alarm1.set_client(vtimer1);
    static_init!(timer : drivers::timer::TimerDriver<AlarmToTimer<'static,
                                VirtualMuxAlarm<'static, nrf51822::rtc::Rtc>>> =
                            drivers::timer::TimerDriver::new(vtimer1,
                                                 process::Container::create()));
    vtimer1.set_client(timer);

    let firestorm : &'static mut Firestorm = mem::transmute(&mut FIRESTORM_BUF);
    *firestorm = Firestorm {
        chip: nrf51822::chip::Nrf51822::new(),
        gpio: gpio,
        timer: timer,
    };

    firestorm
}

use core::fmt::Arguments;
#[cfg(not(test))]
#[lang="panic_fmt"]
#[no_mangle]
pub unsafe extern fn rust_begin_unwind(_args: &Arguments,
    _file: &'static str, _line: usize) -> ! {
    use support::nop;
    use hil::gpio::GPIOPin;

    let led0 = &nrf51822::gpio::PA[18];
    let led1 = &nrf51822::gpio::PA[19];

    led0.enable_output();
    led1.enable_output();
    loop {
        for _ in 0..100000 {
            led0.set();
            led1.set();
            nop();
        }
        for _ in 0..100000 {
            led0.clear();
            led1.clear();
            nop();
        }
    }
}

