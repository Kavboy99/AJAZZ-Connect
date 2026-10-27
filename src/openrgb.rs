//! Independent implementation of the published OpenRGB SDK, protocol levels 1..3.
//! Loopback only. No SaveMode, firmware commands, scanning or server launch.
use crate::firmware_assistant::digest;
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    time::Duration,
};

const LIMIT: usize = 4 * 1024 * 1024;
const DIRECT_FLAGS: u32 = 1 << 5;
const AUTO_SAVE: u32 = 1 << 9;

#[derive(Clone, Debug)]
pub struct Mode {
    pub name: String,
    pub flags: u32,
    wire: Vec<u8>,
    color_mode_at: usize,
    fixed_prefix: usize,
}
#[derive(Clone, Debug)]
pub struct Device {
    pub index: u32,
    pub identity: String,
    pub name: String,
    pub vendor: String,
    pub kind: u32,
    pub led_count: usize,
    pub active: u32,
    pub modes: Vec<Mode>,
    pub colors: Vec<[u8; 4]>,
}

fn parse_mode(c: &mut Cursor<'_>, version: u32) -> Result<Mode> {
    let start = c.at;
    let name = c.string()?;
    c.u32()?;
    let flags = c.u32()?;
    let low = c.u32()?;
    let high = c.u32()?;
    let (b_low, b_high) = if version >= 3 {
        (c.u32()?, c.u32()?)
    } else {
        (0, 0)
    };
    let min_colors = c.u32()?;
    let max_colors = c.u32()?;
    let fixed_prefix = c.at - start;
    let speed = c.u32()?;
    let brightness = if version >= 3 { c.u32()? } else { 0 };
    let direction = c.u32()?;
    let color_mode_at = c.at - start;
    let color_mode = c.u32()?;
    let n = c.count(4096)?;
    c.take(n * 4)?;
    ensure!(
        flags & 1 == 0 || (low <= speed && speed <= high),
        "Скорость режима вне диапазона"
    );
    ensure!(
        version < 3 || flags & 16 == 0 || (b_low <= brightness && brightness <= b_high),
        "Яркость режима вне диапазона"
    );
    ensure!(
        direction <= 9 && color_mode <= 3 && min_colors <= max_colors,
        "Некорректный режим SDK"
    );
    ensure!(
        n as u32 <= max_colors || max_colors == 0 && n == 0,
        "Слишком много цветов режима"
    );
    Ok(Mode {
        name,
        flags,
        wire: c.data[start..c.at].to_vec(),
        color_mode_at,
        fixed_prefix,
    })
}
impl Device {
    pub fn direct(&self) -> Option<usize> {
        self.modes.iter().position(|m| {
            m.name.eq_ignore_ascii_case("direct")
                && m.flags & DIRECT_FLAGS != 0
                && m.flags & AUTO_SAVE == 0
        })
    }
    pub fn writable(&self) -> bool {
        self.kind == 5
            && self.led_count > 0
            && self.colors.len() == self.led_count
            && self.direct().is_some()
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Backup {
    pub schema: u32,
    pub created_utc: String,
    pub device_identity: String,
    pub device_name: String,
    pub protocol: u32,
    pub mode_index: u32,
    pub mode_wire: Vec<u8>,
    pub colors: Vec<[u8; 4]>,
}

struct Cursor<'a> {
    data: &'a [u8],
    at: usize,
}
impl<'a> Cursor<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self.at.checked_add(n).context("SDK offset overflow")?;
        let bytes = self
            .data
            .get(self.at..end)
            .context("Обрезанный пакет SDK")?;
        self.at = end;
        Ok(bytes)
    }
    fn u16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into()?))
    }
    fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into()?))
    }
    fn string(&mut self) -> Result<String> {
        let n = self.u16()? as usize;
        ensure!((1..=4096).contains(&n), "Некорректная строка SDK");
        let bytes = self.take(n)?;
        ensure!(bytes[n - 1] == 0, "Строка SDK без NUL");
        Ok(String::from_utf8_lossy(&bytes[..n - 1]).into_owned())
    }
    fn count(&mut self, max: usize) -> Result<usize> {
        let n = self.u16()? as usize;
        ensure!(n <= max, "Слишком большой список SDK");
        Ok(n)
    }
}

pub fn parse_device(data: &[u8], version: u32, index: u32) -> Result<Device> {
    ensure!(
        (1..=3).contains(&version) && data.len() <= LIMIT,
        "Версия или размер SDK не поддерживается"
    );
    let mut c = Cursor { data, at: 0 };
    ensure!(
        c.u32()? as usize == data.len(),
        "Размер описания SDK не совпал"
    );
    let kind = c.u32()?;
    let name = c.string()?;
    let vendor = c.string()?;
    let description = c.string()?;
    let firmware = c.string()?;
    let serial = c.string()?;
    let location = c.string()?;
    let mode_count = c.count(256)?;
    let active = c.u32()?;
    ensure!(
        mode_count == 0 || (active as usize) < mode_count,
        "Некорректный активный режим"
    );
    let mut modes = Vec::new();
    for _ in 0..mode_count {
        modes.push(parse_mode(&mut c, version)?);
    }
    let zones = c.count(1024)?;
    for _ in 0..zones {
        c.string()?;
        c.take(16)?;
        let matrix = c.u16()? as usize;
        if matrix != 0 {
            ensure!(
                matrix >= 8 && (matrix - 8).is_multiple_of(4),
                "Неверная матрица SDK"
            );
            let bytes = c.take(matrix)?;
            let h = u32::from_le_bytes(bytes[..4].try_into()?) as usize;
            let w = u32::from_le_bytes(bytes[4..8].try_into()?) as usize;
            ensure!(
                h.checked_mul(w)
                    .and_then(|n| n.checked_mul(4))
                    .and_then(|n| n.checked_add(8))
                    == Some(matrix),
                "Размер матрицы SDK не совпал"
            );
        }
    }
    let led_count = c.count(8192)?;
    let mut led_names = Vec::new();
    for _ in 0..led_count {
        led_names.push(c.string()?);
        c.u32()?;
    }
    let n = c.count(8192)?;
    let colors = c.take(n * 4)?.as_chunks::<4>().0.to_vec();
    ensure!(c.at == data.len(), "Неизвестные поля в описании SDK");
    // Hash private identifiers; neither serial nor USB location is exported.
    let identity = digest(&serde_json::to_vec(&(
        kind,
        &name,
        &vendor,
        &description,
        &firmware,
        &serial,
        &location,
        &led_names,
        modes.iter().map(|m| (&m.name, m.flags)).collect::<Vec<_>>(),
    ))?);
    Ok(Device {
        index,
        identity,
        name,
        vendor,
        kind,
        led_count,
        active,
        modes,
        colors,
    })
}

pub struct Client {
    stream: TcpStream,
    pub version: u32,
    pub devices: Vec<Device>,
    pub backups: Vec<Backup>,
}
impl Client {
    pub fn connect(port: u16) -> Result<Self> {
        let address = SocketAddr::from(([127, 0, 0, 1], port));
        let stream = TcpStream::connect_timeout(&address, Duration::from_secs(2))
            .context("OpenRGB SDK недоступен. В OpenRGB включите SDK Server на 127.0.0.1:6742")?;
        stream.set_read_timeout(Some(Duration::from_secs(3)))?;
        stream.set_write_timeout(Some(Duration::from_secs(3)))?;
        stream.set_nodelay(true)?;
        let mut s = Self {
            stream,
            version: 3,
            devices: Vec::new(),
            backups: Vec::new(),
        };
        s.send(0, 40, &3u32.to_le_bytes())?;
        let reply = s.receive(0, 40)?;
        ensure!(reply.len() == 4, "Некорректная версия SDK");
        let server = u32::from_le_bytes(reply.try_into().unwrap());
        ensure!(server >= 1, "Нужен OpenRGB с SDK protocol >= 1");
        s.version = server.min(3);
        s.send(0, 50, b"AJAZZ Connect\0")?;
        let count = s.count()?;
        for index in 0..count {
            let d = s.device(index)?;
            if d.kind == 5 {
                s.devices.push(d);
            }
        }
        Ok(s)
    }
    fn send(&mut self, device: u32, packet: u32, payload: &[u8]) -> Result<()> {
        ensure!(payload.len() <= LIMIT, "SDK packet too large");
        let mut wire = Vec::with_capacity(16 + payload.len());
        wire.extend_from_slice(b"ORGB");
        for n in [device, packet, payload.len() as u32] {
            wire.extend_from_slice(&n.to_le_bytes());
        }
        wire.extend_from_slice(payload);
        self.stream.write_all(&wire)?;
        Ok(())
    }
    fn receive(&mut self, device: u32, packet: u32) -> Result<Vec<u8>> {
        let mut header = [0u8; 16];
        self.stream.read_exact(&mut header)?;
        ensure!(&header[..4] == b"ORGB", "Неверная сигнатура SDK");
        let id = u32::from_le_bytes(header[4..8].try_into()?);
        let kind = u32::from_le_bytes(header[8..12].try_into()?);
        let n = u32::from_le_bytes(header[12..16].try_into()?) as usize;
        ensure!(n <= LIMIT, "SDK packet exceeds limit");
        if kind == 100 {
            bail!("Список OpenRGB изменился. Подключитесь заново; управление остановлено");
        }
        ensure!(
            id == device && kind == packet,
            "Неожиданный пакет SDK {id}:{kind}"
        );
        let mut payload = vec![0; n];
        self.stream.read_exact(&mut payload)?;
        Ok(payload)
    }
    fn count(&mut self) -> Result<u32> {
        self.send(0, 0, &[])?;
        let data = self.receive(0, 0)?;
        ensure!(data.len() == 4, "Некорректный счётчик SDK");
        let n = u32::from_le_bytes(data.as_slice().try_into()?);
        ensure!(n <= 128, "Слишком много устройств SDK");
        Ok(n)
    }
    fn device(&mut self, index: u32) -> Result<Device> {
        self.send(index, 1, &self.version.to_le_bytes())?;
        parse_device(&self.receive(index, 1)?, self.version, index)
    }
    fn checked(&mut self, selected: usize) -> Result<Device> {
        let old = self
            .devices
            .get(selected)
            .context("Выберите клавиатуру")?
            .clone();
        let count = self.count()?;
        ensure!(old.index < count, "Клавиатура отключена");
        let fresh = self.device(old.index)?;
        ensure!(
            old.identity == fresh.identity,
            "Индекс SDK теперь принадлежит другому устройству. Подключитесь заново"
        );
        ensure!(
            fresh.writable(),
            "Клавиатура не предоставляет режим Direct без автоматического сохранения"
        );
        Ok(fresh)
    }
    fn mode(&mut self, index: u32, mode_index: u32, wire: &[u8]) -> Result<()> {
        let mut p = Vec::new();
        p.extend_from_slice(&((8 + wire.len()) as u32).to_le_bytes());
        p.extend_from_slice(&mode_index.to_le_bytes());
        p.extend_from_slice(wire);
        self.send(index, 1101, &p)
    }
    fn colors(&mut self, index: u32, colors: &[[u8; 4]]) -> Result<()> {
        ensure!(colors.len() <= 8192, "Слишком много цветов");
        let mut p = Vec::new();
        p.extend_from_slice(&((6 + 4 * colors.len()) as u32).to_le_bytes());
        p.extend_from_slice(&(colors.len() as u16).to_le_bytes());
        for color in colors {
            p.extend_from_slice(color);
        }
        self.send(index, 1050, &p)
    }
    pub fn solid(&mut self, selected: usize, rgb: [u8; 3]) -> Result<String> {
        let d = self.checked(selected)?;
        if !self.backups.iter().any(|b| b.device_identity == d.identity) {
            let backup = Backup {
                schema: 1,
                created_utc: chrono::Utc::now().to_rfc3339(),
                device_identity: d.identity.clone(),
                device_name: d.name.clone(),
                protocol: self.version,
                mode_index: d.active,
                mode_wire: d.modes[d.active as usize].wire.clone(),
                colors: d.colors.clone(),
            };
            let directory = std::env::var_os("LOCALAPPDATA")
                .map(std::path::PathBuf::from)
                .context("LOCALAPPDATA недоступна: невозможно сохранить резервную копию")?
                .join("AjazzConnect")
                .join("backups");
            std::fs::create_dir_all(&directory)?;
            let path = directory.join(format!(
                "{}-{}.json",
                chrono::Utc::now().format("%Y%m%d-%H%M%S-%f"),
                &d.identity[..16]
            ));
            crate::storage::write_json_atomic(&path, &backup)?;
            self.backups.push(backup);
        }
        let direct = d.direct().context("Нет Direct")?;
        let mut wire = d.modes[direct].wire.clone();
        let at = d.modes[direct].color_mode_at;
        wire[at..at + 4].copy_from_slice(&1u32.to_le_bytes());
        self.mode(d.index, direct as u32, &wire)?;
        let colors = vec![[rgb[0], rgb[1], rgb[2], 0]; d.led_count];
        self.colors(d.index, &colors)?;
        let after = self.device(d.index)?;
        ensure!(
            after.identity == d.identity && after.active == direct as u32 && after.colors == colors,
            "Сервер не подтвердил новое состояние. Резервная копия сохранена"
        );
        Ok("OpenRGB подтвердил цвет в своём состоянии. Физический результат проверьте на клавиатуре.".into())
    }
    pub fn restore(&mut self, selected: usize) -> Result<String> {
        let d = self.devices.get(selected).context("Выберите клавиатуру")?;
        let b = self
            .backups
            .iter()
            .find(|b| b.device_identity == d.identity)
            .context("Для этой клавиатуры нет резервной копии в текущем подключении")?
            .clone();
        self.restore_backup(&b)?;
        Ok("Исходный режим и цвета восстановлены в OpenRGB.".into())
    }
    pub fn restore_backup(&mut self, b: &Backup) -> Result<()> {
        ensure!(
            b.schema == 1 && b.protocol == self.version,
            "Версия резервной копии не совпала"
        );
        let selected = self
            .devices
            .iter()
            .position(|d| d.identity == b.device_identity)
            .context("Устройство резервной копии не найдено")?;
        let d = self.checked(selected)?;
        ensure!(
            b.colors.len() == d.led_count && (b.mode_index as usize) < d.modes.len(),
            "Неверная резервная копия"
        );
        // Accept only the exact server-origin mode definition, allowing runtime parameters to differ.
        let mut c = Cursor {
            data: &b.mode_wire,
            at: 0,
        };
        let mode = parse_mode(&mut c, self.version)?;
        let current = &d.modes[b.mode_index as usize];
        ensure!(
            c.at == b.mode_wire.len()
                && mode.fixed_prefix == current.fixed_prefix
                && mode.wire[..mode.fixed_prefix] == current.wire[..current.fixed_prefix],
            "Режим резервной копии не совпал"
        );
        // Update color buffer while Direct is still active, then restore hardware effect.
        self.colors(d.index, &b.colors)?;
        self.mode(d.index, b.mode_index, &b.mode_wire)?;
        let after = self.device(d.index)?;
        ensure!(
            after.identity == d.identity
                && after.active == b.mode_index
                && after.colors == b.colors,
            "Сервер не подтвердил восстановление"
        );
        Ok(())
    }
}
