#![no_std]
#![no_main]

extern crate alloc;
use esp_backtrace as _;
use esp_println::println;
use esp_wifi::wifi::WifiMode;
use hal::{clock::ClockControl, peripherals::Peripherals, prelude::*, timer::TimerGroup, Rtc, IO, Delay};
use hal::i2c::I2C;
use hal::systimer::SystemTimer;

const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("PASSWORD");

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

fn init_heap() {
    const HEAP_SIZE: usize = 32 * 1024;

    extern "C" {
        static mut _heap_start: u32;
    }

    unsafe {
        let heap_start = &_heap_start as *const _ as usize;
        ALLOCATOR.init(heap_start as *mut u8, HEAP_SIZE);
    }
}

#[entry]
fn main() -> ! {
    println!("Its alive!");
    init_heap();
    let peripherals = Peripherals::take();
    let mut system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // Disable the RTC and TIMG watchdog timers
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(
        peripherals.TIMG0,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(
        peripherals.TIMG1,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt1 = timer_group1.wdt;
    rtc.swd.disable();
    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    println!("Setup complete");

    // Wifi stuff
    {
        let timer = SystemTimer::new(peripherals.SYSTIMER).alarm0;
        let init = esp_wifi::initialize(
            esp_wifi::EspWifiInitFor::Wifi,
            timer,
            hal::Rng::new(peripherals.RNG),
            system.radio_clock_control,
            &clocks,
        ).unwrap();
        let (wifi, _) = peripherals.RADIO.split();

        let (interface, controller) = esp_wifi::wifi::new_with_mode(
                &init,
                wifi,
            WifiMode::Sta
        ).unwrap();
    }

    // Measurement stuff
    // {
    //     let  i2c = I2C::new(
    //         peripherals.I2C0,
    //         io.pins.gpio6,
    //         io.pins.gpio7,
    //         100u32.kHz(),
    //         &mut system.peripheral_clock_control,
    //         &clocks,
    //     );
    //     let delay = Delay::new(&clocks);
    //
    //     let mut aht20 = aht20::Aht20::new(i2c, delay).unwrap();
    //
    //     let mut loop_delay = Delay::new(&clocks);
    //
    //     aht20.reset().unwrap();
    //     aht20.calibrate().unwrap();
    //
    //     while let Ok((humidity, temperature)) = aht20.read() {
    //         println!("Temp {:.1}% {:.1}°C", humidity.rh(), temperature.celsius());
    //         loop_delay.delay_ms(1000u32);
    //     }
    // }

    loop {}
}
