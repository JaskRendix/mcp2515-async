use embedded_hal_async::spi::{ErrorType, Operation, SpiDevice};
use mcp2515_async::{Bitrate, BitrateConfig, Clock, Mcp2515, registers::OperationMode};

#[derive(Debug)]
struct MockError;

impl core::fmt::Display for MockError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "MockError")
    }
}
impl std::error::Error for MockError {}
impl embedded_hal::spi::Error for MockError {
    fn kind(&self) -> embedded_hal::spi::ErrorKind {
        embedded_hal::spi::ErrorKind::Other
    }
}

// Minimal SPI mock compatible with your driver
#[derive(Debug)]
struct MockSpi;

impl ErrorType for MockSpi {
    type Error = MockError;
}

impl SpiDevice<u8> for MockSpi {
    async fn write(&mut self, _data: &[u8]) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn transfer(&mut self, out: &mut [u8], _input: &[u8]) -> Result<(), Self::Error> {
        out.fill(0);
        Ok(())
    }

    async fn transaction(&mut self, _ops: &mut [Operation<'_, u8>]) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let spi = MockSpi;
    let mut can = Mcp2515::new(spi);

    // Reset and configure bitrate
    can.reset().await.unwrap();
    can.set_mode(OperationMode::Config).await.unwrap();
    can.set_bitrate(BitrateConfig::new(Clock::MHz16, Bitrate::Kbps500).unwrap())
        .await
        .unwrap();
    can.set_mode(OperationMode::Normal).await.unwrap();

    // Frame to repeatedly transmit
    let frame = mcp2515_async::can::CanFrame::new(0x123, &[1, 2, 3, 4]).unwrap();

    println!("Starting CAN stress test: flooding TX and monitoring error counters...");

    for i in 0..5 {
        // Transmit frame
        can.send(&frame).await.unwrap();

        // Read error counters
        let tec = can.read_tec().await.unwrap();
        let rec = can.read_rec().await.unwrap();

        println!("Iteration {}: TEC={} REC={}", i + 1, tec, rec);
    }

    println!("Stress test mock completed successfully.");
}
