//! Metal acceleration for M4 Max

use metal::*;

/// Metal acceleration context
pub struct MetalContext {
    pub device: Device,
    pub queue: CommandQueue,
}

impl MetalContext {
    #[cfg(target_os = "macos")]
    pub fn new() -> Option<Self> {
        let device = Device::system_default()?;
        let queue = device.new_command_queue();
        Some(Self { device, queue })
    }
    
    #[cfg(not(target_os = "macos"))]
    pub fn new() -> Option<Self> {
        None
    }
}