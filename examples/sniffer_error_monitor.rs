use core::convert::Infallible;
use embedded_hal::digital::ErrorType as DigitalErrorType;
use embedded_hal_async::digital::Wait;
use embedded_hal_async::spi::{ErrorType, Operation, SpiDevice};
use mcp2515_async::{Bitrate, BitrateConfig, Clock, Mcp2515, registers::OperationMode};

#[derive(Debug)]
struct MockSpi;

impl ErrorType for MockSpi {
    type Error = Infallible;
}

impl SpiDevice<u8> for MockSpi {
    async fn transaction(&mut self, _ops: &mut [Operation<'_, u8>]) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Debug)]
struct MockInt;

impl DigitalErrorType for MockInt {
    type Error = Infallible;
}

impl Wait for MockInt {
    async fn wait_for_high(&mut self) -> Result<(), Self::Error> {
        core::future::pending().await
    }

    async fn wait_for_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn wait_for_rising_edge(&mut self) -> Result<(), Self::Error> {
        core::future::pending().await
    }

    async fn wait_for_falling_edge(&mut self) -> Result<(), Self::Error> {
        core::future::pending().await
    }

    async fn wait_for_any_edge(&mut self) -> Result<(), Self::Error> {
        core::future::pending().await
    }
}

#[tokio::main]
async fn main() {
    let spi = MockSpi;
    let int = MockInt;
    let mut can = Mcp2515::new_with_interrupt(spi, int);

    can.reset().await.unwrap();
    can.set_mode(OperationMode::Config).await.unwrap();
    can.set_bitrate(BitrateConfig::new(Clock::MHz16, Bitrate::Kbps500).unwrap())
        .await
        .unwrap();
    can.set_mode(OperationMode::Normal).await.unwrap();

    println!("CAN sniffer running...");
}
