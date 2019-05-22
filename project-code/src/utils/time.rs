use std;

use mmap::MemoryMap;
use shuteye::sleep;

use super::gpio_driver::mmap_bcm_register;

const TIMER_REGISTER_OFFSET: u64 = 0x3000;
const TIMER_OVERFLOW_VALUE: u32 = 4294967295;



pub struct Timer {
    _timemap: Option<MemoryMap>,
    timereg: *mut u32,
}

impl Timer {
    unsafe fn read(self: &Timer) -> u32 {
        std::ptr::read_volatile(self.timereg)
    }

    pub fn new() -> Timer {
        //TODO: Implement this yourself.
        let map = mmap_bcm_register(TIMER_REGISTER_OFFSET as usize);

        let mut timer: Timer = Timer {
            _timemap: None,
            timereg: 0 as *mut u32,
        };

        match &map {
            &Some(ref map) => {
                unsafe {
                    timer.timereg = map.data() as *mut u32;
                    timer.timereg.offset(1);
                }
            }
            &None => {}
        };
        timer
    }

    pub fn nanosleep(self: &Timer, mut nanos: u32) {
        //TODO: Implement this yourself.
        let k_jitter_allowance = 60 * 1000;

        if nanos > k_jitter_allowance + 5000{
            let before_microsecs = unsafe { self.read() };
            let nanosec_passed: u64;

            match sleep(std::time::Duration::new(0, nanos - k_jitter_allowance)) {
                Some(_time) => {
                    let after_microsec = unsafe { self.read() };
                    if after_microsec > before_microsecs {
                        nanosec_passed = (1000 * (after_microsec - before_microsecs)) as u64;
                    } else {
                        nanosec_passed = 1000 * (TIMER_OVERFLOW_VALUE - before_microsecs + after_microsec) as u64;
                    }
                    if nanosec_passed > nanos as u64 {
                        return;
                    } else {
                        nanos -= nanosec_passed as u32;
                    }
                }
                None => {}
            }
        }

        if nanos < 20 {
            return;
        }

        let start_time: u32 = unsafe { self.read() };
        let mut current_time: u32 = start_time;

        while start_time + nanos * 1000 <= current_time {
            current_time = unsafe { self.read() };
        }
        return;
    }
}


