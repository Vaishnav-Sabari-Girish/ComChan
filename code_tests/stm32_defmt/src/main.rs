#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_probe as _;

use embassy_executor::Spawner;
use embassy_time::Timer;


#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let _p = embassy_stm32::init(Default::default());

    defmt::info!("Hello World from STM32 Nucleo-L433 RC-P");
    loop {
        defmt::info!("Echo...");

        Timer::after_secs(1).await;
    }
}
