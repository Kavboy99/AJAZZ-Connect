//! hidapi wrapper: enumeration into passive `Endpoint`s and short-lived channels.

use super::hid_caps;
use crate::core::{DriverError, Endpoint, HidChannel};
use std::ffi::CString;

pub(super) struct HidHost {
    api: hidapi::HidApi,
}

impl HidHost {
    pub(super) fn enumerate() -> Result<(Self, Vec<Endpoint>), DriverError> {
        let api = hidapi::HidApi::new().map_err(io)?;
        let endpoints = api
            .device_list()
            .map(|info| {
                let path = info.path().to_bytes().to_vec();
                Endpoint::hid(
                    info.vendor_id(),
                    info.product_id(),
                    info.usage_page(),
                    info.usage(),
                )
                .with_interface(info.interface_number())
                .with_release(info.release_number())
                .with_product(info.product_string().unwrap_or_default())
                .with_reports(hid_caps::report_sizes(&info.path().to_string_lossy()))
                .with_path(path)
            })
            .collect();
        Ok((Self { api }, endpoints))
    }

    pub(super) fn open(&self, endpoint: &Endpoint) -> Result<Box<dyn HidChannel>, DriverError> {
        let path = CString::new(endpoint.path())
            .map_err(|_| DriverError::Io("device path contains NUL".into()))?;
        let device = self.api.open_path(&path).map_err(io)?;
        Ok(Box::new(Channel(device)))
    }
}

struct Channel(hidapi::HidDevice);

impl HidChannel for Channel {
    fn write(&self, data: &[u8]) -> Result<usize, DriverError> {
        self.0.write(data).map_err(io)
    }

    fn read_timeout(&self, buffer: &mut [u8], milliseconds: i32) -> Result<usize, DriverError> {
        self.0.read_timeout(buffer, milliseconds).map_err(io)
    }

    fn send_feature(&self, data: &[u8]) -> Result<(), DriverError> {
        self.0.send_feature_report(data).map_err(io)
    }

    fn get_feature(&self, buffer: &mut [u8]) -> Result<usize, DriverError> {
        self.0.get_feature_report(buffer).map_err(io)
    }

    fn report_descriptor(&self, buffer: &mut [u8]) -> Result<usize, DriverError> {
        self.0.get_report_descriptor(buffer).map_err(io)
    }

    fn container_id(&self) -> Result<u128, DriverError> {
        let id = self.0.get_container_id().map_err(io)?;
        let mut bytes = [0u8; 16];
        bytes[..4].copy_from_slice(&id.data1.to_le_bytes());
        bytes[4..6].copy_from_slice(&id.data2.to_le_bytes());
        bytes[6..8].copy_from_slice(&id.data3.to_le_bytes());
        bytes[8..].copy_from_slice(&id.data4);
        Ok(u128::from_le_bytes(bytes))
    }
}

fn io(error: hidapi::HidError) -> DriverError {
    DriverError::Io(error.to_string())
}
