use rusb::{
    Device, DeviceDescriptor, DeviceHandle, GlobalContext
};

pub struct Reader {
    pub descriptor: DeviceDescriptor,
    pub device: Device<GlobalContext>,
    pub handle: DeviceHandle<GlobalContext>
}

pub struct ReaderData {
    pub message_type: u8,
    pub length: u32,
    pub slot: u8,
    pub seq: u8,
    pub status: u8,
    pub error: u8,
    pub chain_parameter: u8,
    pub valid: bool,
    pub id: u64,
    pub adpu_status: u16
}
