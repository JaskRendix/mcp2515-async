use super::mock_spi::MockSpi;
use mcp2515_async::Mcp2515;

pub struct TestDevice {
    pub dev: Mcp2515<MockSpi>,
    pub spi: MockSpi,
}

impl TestDevice {
    pub fn new() -> Self {
        let spi = MockSpi::new();
        let dev = Mcp2515::new(spi.clone());
        Self { dev, spi }
    }

    pub fn with_transfer_output(out: &[u8]) -> Self {
        let spi = MockSpi::new().with_transfer_output(out);
        let dev = Mcp2515::new(spi.clone());
        Self { dev, spi }
    }

    pub fn writes(&self) -> Vec<Vec<u8>> {
        self.spi.writes()
    }
}
