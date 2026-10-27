//! Common stock configuration protocol. No firmware hooks, ISP or memory writer.
//! Only GET_INFO(10), GET_LED_EFFECT(13), SET_LED_EFFECT(23), all at address zero.
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};

pub fn packet(command: u8, payload: Option<&[u8]>) -> Result<[u8; 65]> {
    ensure!(
        matches!(command, 0x10 | 0x13 | 0x23),
        "Команда не разрешена адаптером подсветки"
    );
    let size = if command == 0x10 { 48 } else { 16 };
    ensure!((command == 0x23) == payload.is_some(), "GET/SET mismatch");
    if let Some(p) = payload {
        ensure!(p.len() == size, "Неверная длина подсветки");
    }
    let mut p = [0; 65];
    p[1..9].copy_from_slice(&[0xaa, command, size as u8, 0, 0, 0, 1, 0]);
    if let Some(data) = payload {
        p[9..9 + size].copy_from_slice(data);
    }
    Ok(p)
}
pub fn response(raw: &[u8], command: u8, size: usize) -> Result<Vec<u8>> {
    let p = if raw.first() == Some(&0) {
        &raw[1..]
    } else {
        raw
    };
    ensure!(
        p.len() == 64 && p[..5] == [0x55, command, size as u8, 0, 0],
        "Ответ AA55 не совпал с запросом"
    );
    ensure!(size <= 56, "Слишком большой ответ");
    Ok(p[8..8 + size].to_vec())
}
pub fn validate_lighting(b: &[u8]) -> Result<()> {
    ensure!(
        b.len() == 16
            && b[0] <= 20
            && b[8] <= 1
            && b[9] <= 5
            && b[10] <= 5
            && b[11] <= 3
            && b[14..] == [0xaa, 0x55],
        "Формат LED_EFFECT отличается от исследованного протокола"
    );
    Ok(())
}
pub fn solid_payload(original: &[u8], rgb: [u8; 3]) -> Result<Vec<u8>> {
    validate_lighting(original)?;
    let mut p = original.to_vec();
    p[0] = 1;
    p[1..4].copy_from_slice(&rgb);
    p[4] = 255;
    p[8] = 0;
    if p[9] == 0 {
        p[9] = 5;
    } // User explicitly enables visible static lighting.
    p[14] = 0xaa;
    p[15] = 0x55;
    Ok(p)
}
pub fn identity(info: &[u8]) -> Result<String> {
    crate::protocol::KeyboardInfo::parse(info)?;
    let mut b = info[4..16].to_vec();
    b.push(info[19]);
    b.extend_from_slice(&info[30..33]);
    Ok(crate::firmware_assistant::digest(&b))
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Backup {
    pub schema: u32,
    pub kind: String,
    pub created_utc: String,
    pub device_identity: String,
    pub protocol_identity: String,
    pub lighting: Vec<u8>,
}

#[cfg(windows)]
pub struct Device {
    pub name: String,
    pub info: crate::protocol::KeyboardInfo,
    pub identity: String,
    protocol_identity: String,
    device: hidapi::HidDevice,
    original: Vec<u8>,
    backup: Option<Backup>,
}
#[cfg(windows)]
impl Device {
    fn transaction(
        device: &hidapi::HidDevice,
        command: u8,
        payload: Option<&[u8]>,
    ) -> Result<Vec<u8>> {
        use std::time::{Duration, Instant};
        // Queue a native input read before Output, as in the vendor WebHID transport.
        let mut quiet = false;
        for _ in 0..64 {
            let mut b = [0; 65];
            if device.read_timeout(&mut b, 0)? == 0 {
                quiet = true;
                break;
            }
        }
        ensure!(quiet, "Канал занят другим драйвером; закройте его");
        let p = packet(command, payload)?;
        let n = device.write(&p)?;
        // The Windows native backend may return zero after synchronous completion.
        ensure!(n == 0 || n == p.len(), "Неполный Output Report");
        let size = if command == 0x10 { 48 } else { 16 };
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            let mut b = [0; 65];
            let n = device.read_timeout(&mut b, 100)?;
            if let Ok(data) = response(&b[..n], command, size) {
                return Ok(data);
            }
        }
        anyhow::bail!(
            "Нет подтверждения AA55 команды {command:02X}; повторная запись не выполняется"
        )
    }
    pub fn discover() -> Result<(Vec<Self>, Vec<String>)> {
        let api = hidapi::HidApi::new()?;
        let mut devices = Vec::new();
        let mut notes = Vec::new();
        let _lease = crate::hardware::DeviceLease::acquire("AK820MAX")?;
        for info in api.device_list().filter(|d| {
            d.vendor_id() == 0x0c45
                && d.usage_page() == 0xff68
                && d.usage() == 0x61
                && d.product_id() != 0x7210
                && d.product_string()
                    .is_some_and(|s| s.to_ascii_uppercase().contains("AK820"))
        }) {
            let name = info.product_string().unwrap_or("AK820").to_string();
            let result = (|| -> Result<Self> {
                let shape =
                    crate::firmware_assistant::endpoint_shape(&info.path().to_string_lossy())?;
                ensure!(
                    shape.0 == 65 && shape.1 == 65,
                    "Другой размер канала; AA55 не отправлялся"
                );
                let device = info.open_device(&api)?;
                let raw = Self::transaction(&device, 0x10, None)?;
                let parsed = crate::protocol::KeyboardInfo::parse(&raw)?;
                ensure!(
                    parsed.vid == info.vendor_id() && parsed.pid == info.product_id(),
                    "VID/PID GET_INFO не совпал с USB"
                );
                let protocol_identity = identity(&raw)?;
                let original = Self::transaction(&device, 0x13, None)?;
                validate_lighting(&original)?;
                Ok(Self {
                    name: name.clone(),
                    info: parsed,
                    identity: crate::firmware_assistant::digest(info.path().to_bytes()),
                    protocol_identity,
                    device,
                    original,
                    backup: None,
                })
            })();
            match result {
                Ok(d) => devices.push(d),
                Err(e) => notes.push(format!("{name}: {e:#}")),
            }
        }
        Ok((devices, notes))
    }
    fn check(&self) -> Result<()> {
        let info = Self::transaction(&self.device, 0x10, None)?;
        ensure!(
            identity(&info)? == self.protocol_identity,
            "Устройство или версия прошивки изменилась; подключитесь заново"
        );
        Ok(())
    }
    fn write_lighting(&self, p: &[u8]) -> Result<String> {
        validate_lighting(p)?;
        let mut wire = p.to_vec();
        if wire[0] == 20 {
            wire[0] = 128;
        } // Vendor SET representation of saved per-key mode.
        wire[4] = 255;
        Self::transaction(&self.device, 0x23, Some(&wire))?;
        std::thread::sleep(std::time::Duration::from_millis(200));
        let observed = Self::transaction(&self.device, 0x13, None)?;
        validate_lighting(&observed)?;
        let same = |a: &[u8], b: &[u8]| {
            (0..16)
                .filter(|i| ![4, 14, 15].contains(i))
                .all(|i| a[i] == b[i])
        };
        if same(&observed, p) {
            return Ok("Параметры подсветки подтверждены обратным чтением.".into());
        }
        // A documented 80A0 firmware quirk: stock GET reports its factory palette.
        if self.info.pid == 0x80a0
            && matches!(self.info.version.as_str(), "1.30" | "1.36")
            && p[0] <= 19
            && observed[1..4] == [255, 255, 255]
            && observed[8] == 1
        {
            let mut normalized = p.to_vec();
            normalized[1..4].copy_from_slice(&observed[1..4]);
            normalized[8] = observed[8];
            ensure!(same(&observed, &normalized), "Обратное чтение не совпало");
            return Ok("Режим подтверждён; эта версия GET возвращает стандартную палитру. Проверьте цвет на клавиатуре.".into());
        }
        anyhow::bail!(
            "ACK получен, но обратное чтение не совпало. Резервная копия сохранена; повторная запись не выполнялась"
        )
    }
    pub fn solid(&mut self, rgb: [u8; 3]) -> Result<String> {
        let _lease = crate::hardware::DeviceLease::acquire("AK820MAX")?;
        self.check()?;
        let current = Self::transaction(&self.device, 0x13, None)?;
        let p = solid_payload(&current, rgb)?;
        if self.backup.is_none() {
            self.original = current.clone();
            let b = Backup {
                schema: 1,
                kind: "aa55-lighting".into(),
                created_utc: chrono::Utc::now().to_rfc3339(),
                device_identity: self.identity.clone(),
                protocol_identity: self.protocol_identity.clone(),
                lighting: current,
            };
            let directory = std::env::var_os("LOCALAPPDATA")
                .map(std::path::PathBuf::from)
                .context("LOCALAPPDATA недоступна")?
                .join("AjazzConnect/backups");
            std::fs::create_dir_all(&directory)?;
            crate::storage::write_json_atomic(
                &directory.join(format!(
                    "aa55-{}-{}.json",
                    chrono::Utc::now().format("%Y%m%d-%H%M%S-%f"),
                    &self.identity[..16]
                )),
                &b,
            )?;
            self.backup = Some(b);
        }
        self.write_lighting(&p)
    }
    pub fn restore(&self) -> Result<String> {
        ensure!(
            self.backup.is_some(),
            "В текущем подключении подсветка ещё не менялась"
        );
        let _lease = crate::hardware::DeviceLease::acquire("AK820MAX")?;
        self.check()?;
        self.write_lighting(&self.original)
    }
    pub fn restore_backup(&self, b: &Backup) -> Result<String> {
        ensure!(
            b.schema == 1
                && b.kind == "aa55-lighting"
                && b.device_identity == self.identity
                && b.protocol_identity == self.protocol_identity,
            "Резервная копия относится к другому устройству"
        );
        let _lease = crate::hardware::DeviceLease::acquire("AK820MAX")?;
        self.check()?;
        self.write_lighting(&b.lighting)
    }
}
