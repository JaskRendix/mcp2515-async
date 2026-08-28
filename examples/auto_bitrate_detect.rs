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
        out.fill(0);
        Ok(())
    }

    async fn transaction(&mut self, _ops: &mut [Operation<'_, u8>]) -> Result<(), Self::Error> {
        Ok(())
    }
}

// Bitrates to try
const RATES: &[(Clock, Bitrate)] = &[
    (Clock::MHz16, Bitrate::Kbps125),
    (Clock::MHz16, Bitrate::Kbps250),
    (Clock::MHz16, Bitrate::Kbps500),
    (Clock::MHz16, Bitrate::Mbps1),
];

#[tokio::main]
async fn main() {
    let spi = MockSpi;
    let mut can = Mcp2515::new(spi);

    // Reset and enter configuration mode
    can.reset().await.unwrap();
    can.set_mode(OperationMode::Config).await.unwrap();

    // Try each bitrate until a frame is successfully received
    for (clk, br) in RATES {
        println!("Trying bitrate {:?}", br);

        if let Some(cfg) = BitrateConfig::new(*clk, *br) {
            can.set_bitrate(cfg).await.unwrap();
        }

        // Listen-only mode: receive frames without affecting the bus
        can.set_mode(OperationMode::ListenOnly).await.unwrap();

        match can.read().await {
            Ok(frame) => {
                println!("Locked onto bitrate {:?}!", br);
                println!("First frame: {:?}", frame);
                return;
            }
            Err(_) => {
                println!("No frame at {:?}, trying next...", br);
                // Switch back to Config mode before setting next bitrate
                can.set_mode(OperationMode::Config).await.unwrap();
                continue;
            }
        }
    }

    println!("No bitrate matched.");
}
