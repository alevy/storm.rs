use core::prelude::*;
use core::intrinsics;
use super::nvic;
use hil::timer::Timer;

#[repr(C, packed)]
#[allow(missing_copy_implementations)]
struct AstRegisters {
    cr: u32,
    cv: u32,
    sr: u32,
    scr: u32,
    ier: u32,
    idr: u32,
    imr: u32,
    wer: u32,
    //0x20
    ar0: u32,
    ar1: u32,
    reserved0: [u32; 2],
    pir0: u32,
    pir1: u32,
    reserved1: [u32; 2],
    //0x40
    clock: u32,
    dtr: u32,
    eve: u32,
    evd: u32,
    evm: u32,
    calv: u32
    //we leave out parameter and version
}

pub const AST_BASE: isize = 0x400F0800;

#[allow(missing_copy_implementations)]
pub struct Ast<F: Fn()> {
    addr: *mut AstRegisters,
    callback: Option<F>
}

#[repr(uint)]
pub enum Clock {
    ClockRCSys = 0,
    ClockOsc32 = 1,
    ClockAPB = 2,
    ClockGclk2 = 3,
    Clock1K = 4
}

impl Copy for Clock {}

impl<F: Fn()> Ast<F> {
    pub fn new() -> Ast<F> {
        Ast {addr: AST_BASE as *mut AstRegisters, callback: None}
    }

    pub fn setup(&mut self) {
        self.select_clock(Clock::ClockRCSys);
        self.set_prescalar(0);
        self.clear_alarm();
    }

    pub fn clock_busy(&self) -> bool {
        unsafe {
            intrinsics::volatile_load(&(*self.addr).sr) & (1 << 28) != 0
        }
    }


    pub fn busy(&self) -> bool {
        unsafe {
            intrinsics::volatile_load(&(*self.addr).sr) & (1 << 24) != 0
        }
    }

    // Clears the alarm bit in the status register (indicating the alarm value
    // has been reached).
    pub fn clear_alarm(&mut self) {
        while self.busy() {}
        unsafe {
            intrinsics::volatile_store(&mut (*self.addr).scr, 1 << 8);
        }
    }

    // Clears the per0 bit in the status register (indicating the alarm value
    // has been reached).
    pub fn clear_periodic(&mut self) {
        while self.busy() {}
        unsafe {
            intrinsics::volatile_store(&mut (*self.addr).scr, 1 << 16);
        }
    }


    pub fn select_clock(&mut self, clock: Clock) {
        unsafe {
          // Disable clock by setting first bit to zero
          let enb = intrinsics::volatile_load(&(*self.addr).clock) ^ 1;
          intrinsics::volatile_store(&mut (*self.addr).clock, enb);
          while self.clock_busy() {}

          // Select clock
          intrinsics::volatile_store(&mut (*self.addr).clock, (clock as u32) << 8);
          while self.clock_busy() {}

          // Re-enable clock
          let enb = intrinsics::volatile_load(&(*self.addr).clock) | 1;
          intrinsics::volatile_store(&mut (*self.addr).clock, enb);
        }
    }

    pub fn enable(&mut self) {
        while self.busy() {}
        unsafe {
            let cr = intrinsics::volatile_load(&(*self.addr).cr) | 1;
            intrinsics::volatile_store(&mut (*self.addr).cr, cr);
        }
    }

    pub fn disable(&mut self) {
        while self.busy() {}
        unsafe {
            let cr = intrinsics::volatile_load(&(*self.addr).cr) & !1;
            intrinsics::volatile_store(&mut (*self.addr).cr, cr);
        }
    }

    pub fn set_prescalar(&mut self, val: u8) {
        while self.busy() {}
        unsafe {
            let cr = intrinsics::volatile_load(&(*self.addr).cr) | (val as u32) << 16;
            intrinsics::volatile_store(&mut (*self.addr).cr, cr);
        }
    }

    pub fn enable_alarm_irq(&mut self) {
        nvic::enable(nvic::NvicIdx::ASTALARM);
        unsafe {
            intrinsics::volatile_store(&mut (*self.addr).ier, 1 << 8);
        }
    }

    pub fn disable_alarm_irq(&mut self) {
        unsafe {
            intrinsics::volatile_store(&mut (*self.addr).idr, 1 << 8);
        }
    }

    pub fn enable_ovf_irq(&mut self) {
        nvic::enable(nvic::NvicIdx::ASTOVF);
        unsafe {
            intrinsics::volatile_store(&mut (*self.addr).ier, 1);
        }
    }

    pub fn disable_ovf_irq(&mut self) {
        unsafe {
            intrinsics::volatile_store(&mut (*self.addr).idr, 1);
        }
    }

    pub fn enable_periodic_irq(&mut self) {
        nvic::enable(nvic::NvicIdx::ASTPER);
        unsafe {
            intrinsics::volatile_store(&mut (*self.addr).ier, 1 << 16);
        }
    }

    pub fn disable_periodic_irq(&mut self) {
        unsafe {
            intrinsics::volatile_store(&mut (*self.addr).idr, 1 << 16);
        }
    }

    pub fn set_periodic_interval(&mut self, interval: u32) {
        while self.busy() {}
        unsafe {
            intrinsics::volatile_store(&mut (*self.addr).pir0, interval);
        }
    }

    pub fn get_counter(&self) -> u32 {
        while self.busy() {}
        unsafe {
            intrinsics::volatile_load(&(*self.addr).cv)
        }
    }


    pub fn set_counter(&mut self, value: u32) {
        while self.busy() {}
        unsafe {
            intrinsics::volatile_store(&mut (*self.addr).cv, value);
        }
    }

    pub fn interrupt_fired(&mut self) {
        self.clear_alarm();
        self.callback.as_ref().map(|f| {
            f()
        });
    }

    pub fn set_callback(&mut self, callback: F)
            where F: Fn() {
        self.callback = Some(callback);
    }
}

impl<F: Fn()> Timer for Ast<F> {
    fn now(&self) -> u32 {
        unsafe {
            intrinsics::volatile_load(&(*self.addr).cv)
        }
    }

    fn disable_alarm(&mut self) {
        self.disable();
        self.clear_alarm();
    }

    fn set_alarm(&mut self, tics: u32) {
        self.disable();
        self.clear_alarm();
        self.enable_alarm_irq();
        while self.busy() {}
        unsafe {
            intrinsics::volatile_store(&mut (*self.addr).ar0, tics);
        }
        self.enable();
    }
}

/*#[no_mangle]
#[allow(non_snake_case)]
pub extern fn AST_ALARM_Handler() {
    unsafe { let f = Ast0.callback; f() }
}*/

