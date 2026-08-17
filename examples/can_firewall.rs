use embedded_hal_async::spi::{ErrorType, Operation, SpiDevice};
use mcp2515_async::{
    config::{AcceptanceFilter, FilterMask},
    registers::OperationMode,
    Bitrate, BitrateConfig, Clock, Mcp2515,
};

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

// Minimal SPI mock that compiles and works with your driver
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
        // Fill with zeros — real hardware would return actual RX buffer contents
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

    // Accept only IDs 0x100–0x1FF
    can.set_filter_mask(FilterMask::Mask0, false, 0x700)
        .await
        .unwrap();

    // Filter: accept IDs starting at 0x100
    can.set_filter(AcceptanceFilter::Rxf0, false, 0x100)
        .await
        .unwrap();

    // Normal operation mode
    can.set_mode(OperationMode::Normal).await.unwrap();

    println!("CAN firewall active: allowing 0x100–0x1FF, blocking everything else.");

    let loop_count = 5; // Limit loop for mock execution
    let mut iterations = 0;

    loop {
        let frame = match can.read().await {
            Ok(f) => f,
            Err(_) => {
                iterations += 1;
                if iterations >= loop_count {
                    println!("Mock execution finished.");
                    break;
                }
                continue;
            }
        };

        let id = frame.standard_id();

        if (id & 0x700) == 0x100 {
            println!("ALLOWED: ID=0x{:03X} DATA={:?}", id, frame.data);
        } else {
            println!("BLOCKED: ID=0x{:03X}", id);
        }
    }
}
