use mcp2515_async::{
    config::{AcceptanceFilter, Bitrate, BitrateConfig, Clock, FilterMask},
    registers::OperationMode,
    Mcp2515,
};

struct MockSpi;

impl embedded_hal_async::spi::ErrorType for MockSpi {
    type Error = core::convert::Infallible;
}

impl embedded_hal_async::spi::SpiDevice<u8> for MockSpi {
    async fn transaction(
        &mut self,
        _operations: &mut [embedded_hal_async::spi::Operation<'_, u8>],
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let spi = MockSpi;
    let mut mcp = Mcp2515::new(spi);

    // Reset and configure bitrate
    mcp.reset().await.unwrap();
    mcp.set_mode(OperationMode::Config).await.unwrap();

    let bitrate_config = BitrateConfig::new(Clock::MHz16, Bitrate::Kbps500).unwrap();
    mcp.set_bitrate(bitrate_config).await.unwrap();

    // Setup hardware acceptance filters
    mcp.set_filter_mask(FilterMask::Mask0, false, 0x7FF)
        .await
        .unwrap();
    mcp.set_filter(AcceptanceFilter::Rxf0, false, 0x100)
        .await
        .unwrap();
    mcp.set_filter(AcceptanceFilter::Rxf1, false, 0x200)
        .await
        .unwrap();

    // Switch to Normal operational mode
    mcp.set_mode(OperationMode::Normal).await.unwrap();

    println!("Gateway bridge example initialized successfully.");
    Ok(())
}
