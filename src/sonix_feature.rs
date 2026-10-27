//! Classic SONiX Feature protocol. Experimental normal lighting settings only.
//! Facts: official-driver RE in OpenKey; author's owner test 2026-10-11.
//! No firmware download, per-key table, macros, TFT or streaming commands.
use anyhow::{Result, ensure};

pub fn command(opcode: u8, argument: u8) -> Result<[u8; 65]> {
    ensure!(
        matches!(opcode, 0x18 | 0x13 | 0x02 | 0xf0),
        "Команда не разрешена адаптером"
    );
    ensure!(
        argument == 0 || opcode == 0x13 && argument == 1,
        "Неверный аргумент"
    );
    let mut p = [0; 65];
    p[1] = 4;
    p[2] = opcode;
    p[9] = argument;
    Ok(p)
}
pub fn ack(packet: &[u8], opcode: u8) -> Result<()> {
    ensure!(
        packet.len() == 65
            && packet[0] == 0
            && packet[1] == 4
            && packet[2] == opcode
            && packet[4] == 1,
        "Нет точного подтверждения команды SONiX {opcode:02X}"
    );
    Ok(())
}
pub fn solid(rgb: [u8; 3]) -> [u8; 65] {
    let mut p = [0; 65];
    p[1] = 1;
    p[2..5].copy_from_slice(&rgb);
    p[10] = 5;
    p[11] = 3;
    p[15] = 0xaa;
    p[16] = 0x55;
    p
}

pub fn apply_transaction(
    mut send: impl FnMut(&[u8; 65], bool) -> Result<()>,
    rgb: [u8; 3],
) -> Result<()> {
    send(&command(0x18, 0)?, true)?;
    let result = (|| -> Result<()> {
        send(&command(0x13, 1)?, true)?;
        send(&solid(rgb), false)?;
        send(&command(2, 0)?, true)?;
        Ok(())
    })();
    let finish = send(&command(0xf0, 0)?, false);
    result?;
    finish
}
#[cfg(windows)]
pub struct Device {
    pub name: String,
    pub release: u16,
    device: hidapi::HidDevice,
}
#[cfg(windows)]
impl Device {
    fn send(&self, p: &[u8; 65], handshake: bool) -> Result<()> {
        std::thread::sleep(std::time::Duration::from_millis(35));
        self.device.send_feature_report(p)?;
        if handshake {
            std::thread::sleep(std::time::Duration::from_millis(35));
            let mut response = [0; 65];
            let n = self.device.get_feature_report(&mut response)?;
            ensure!(n == 65, "Короткий Feature Report");
            ack(&response, p[2])?;
        }
        Ok(())
    }
    fn finish(&self) -> Result<()> {
        self.send(&command(0xf0, 0)?, false)
    }
    pub fn discover() -> Result<(Vec<Self>, Vec<String>)> {
        let api = hidapi::HidApi::new()?;
        let mut devices = Vec::new();
        let mut notes = Vec::new();
        let _lease = crate::hardware::DeviceLease::acquire("AK820MAX")?;
        for info in api.device_list().filter(|i| {
            i.vendor_id() == 0x0c45
                && [0x8009, 0x800a].contains(&i.product_id())
                && i.usage_page() == 0xff13
                && i.usage() == 1
                && ["AK820 MAX", "AK820"].contains(&i.product_string().unwrap_or("").trim())
        }) {
            let name = info.product_string().unwrap_or("AK820").to_string();
            let result = (|| -> Result<Self> {
                let shape =
                    crate::firmware_assistant::endpoint_shape(&info.path().to_string_lossy())?;
                ensure!(shape.2 == 65, "Ожидался Feature Report на 65 байт");
                let device = info.open_device(&api)?;
                let container = device.get_container_id()?;
                // A large data pipe must belong to the same physical USB container.
                let mut partner = false;
                for data in api.device_list().filter(|i| {
                    i.vendor_id() == info.vendor_id()
                        && i.product_id() == info.product_id()
                        && i.usage_page() == 0xff68
                        && i.usage() == 0x61
                }) {
                    let Ok(shape) =
                        crate::firmware_assistant::endpoint_shape(&data.path().to_string_lossy())
                    else {
                        continue;
                    };
                    if shape.1 != 4097 {
                        continue;
                    }
                    let Ok(data_device) = data.open_device(&api) else {
                        continue;
                    };
                    let Ok(other) = data_device.get_container_id() else {
                        continue;
                    };
                    if (
                        container.data1,
                        container.data2,
                        container.data3,
                        container.data4,
                    ) == (other.data1, other.data2, other.data3, other.data4)
                    {
                        partner = true;
                        break;
                    }
                }
                ensure!(
                    partner,
                    "Нет FF68/4096 в том же USB-контейнере; START не отправлялся"
                );
                let d = Self {
                    name: name.clone(),
                    release: info.release_number(),
                    device,
                };
                d.send(&command(0x18, 0)?, true)?;
                d.finish()?;
                Ok(d)
            })();
            match result {
                Ok(d) => devices.push(d),
                Err(e) => notes.push(format!("{name}: {e:#}")),
            }
        }
        Ok((devices, notes))
    }
    pub fn apply(&self, rgb: [u8; 3]) -> Result<String> {
        let _lease = crate::hardware::DeviceLease::acquire("AK820MAX")?;
        apply_transaction(|packet, handshake| self.send(packet, handshake), rgb)?;
        Ok("SONiX подтвердил команды настройки и сохранения. Цвет проверьте на клавиатуре. Исходный режим не считывается этим протоколом.".into())
    }
}
