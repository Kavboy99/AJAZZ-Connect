//! Standard HID Lighting page 0x59. Field order and widths are checked before use.
//! Independently written from USB HID tables / Microsoft reference descriptor.
use anyhow::{Context, Result, ensure};
use std::collections::BTreeMap;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReportMap {
    pub attributes: u8,
    pub request: u8,
    pub response: u8,
    pub multi: u8,
    pub range: u8,
    pub control: u8,
    pub capacity: usize,
}
#[derive(Clone, Copy, Default)]
struct Globals {
    page: u32,
    id: u8,
    bits: u32,
    count: u32,
}
type Fields = Vec<(u16, u32)>;

pub fn parse_descriptor(data: &[u8]) -> Result<ReportMap> {
    ensure!(data.len() <= 16384, "Дескриптор слишком большой");
    let mut at = 0;
    let mut g = Globals::default();
    let mut saved = Vec::new();
    let mut usages: Vec<u32> = Vec::new();
    let mut minimum = None;
    let mut collections: Vec<u32> = Vec::new();
    let mut reports: BTreeMap<u16, (u8, Fields)> = BTreeMap::new();
    let mut id_owner = BTreeMap::new();
    while at < data.len() {
        let tag = data[at];
        at += 1;
        ensure!(tag != 0xfe, "Long HID items не поддерживаются");
        let n = match tag & 3 {
            3 => 4,
            x => x as usize,
        };
        let raw = data.get(at..at + n).context("Обрезанный HID item")?;
        at += n;
        let value = raw
            .iter()
            .enumerate()
            .fold(0u32, |a, (i, b)| a | ((*b as u32) << (i * 8)));
        let key = tag & 0xfc;
        match key {
            0x04 => g.page = value,
            0x74 => g.bits = value,
            0x94 => g.count = value,
            0x84 => {
                ensure!((1..=255).contains(&value), "Нулевой Report ID");
                g.id = value as u8;
            }
            0xa4 => {
                ensure!(saved.len() < 16, "HID Push overflow");
                saved.push(g);
            }
            0xb4 => g = saved.pop().context("HID Pop без Push")?,
            0x08 => usages.push(if n == 4 {
                value
            } else {
                (g.page << 16) | value
            }),
            0x18 => {
                minimum = Some(if n == 4 {
                    value
                } else {
                    (g.page << 16) | value
                })
            }
            0x28 => {
                let first = minimum.take().context("UsageMaximum без Minimum")?;
                let last = if n == 4 {
                    value
                } else {
                    (g.page << 16) | value
                };
                ensure!(
                    last >= first && last - first < 1024,
                    "Слишком большой HID Usage range"
                );
                usages.extend(first..=last);
            }
            0xa0 => {
                ensure!(collections.len() < 32, "Слишком глубокий HID");
                collections.push(usages.first().copied().unwrap_or(0));
            }
            0xc0 => {
                collections.pop().context("EndCollection без Collection")?;
            }
            0xb0 => {
                ensure!(
                    g.count <= 2048 && (1..=32).contains(&g.bits),
                    "Необычные поля Feature"
                );
                let report_usage = collections
                    .iter()
                    .rev()
                    .find(|u| {
                        **u >> 16 == 0x59
                            && matches!(**u & 0xffff, 2 | 0x20 | 0x22 | 0x50 | 0x60 | 0x70)
                    })
                    .copied();
                ensure!(collections.contains(&0x590001), "Feature вне LampArray");
                let usage = (report_usage.context("Feature вне отчёта LampArray")? & 0xffff) as u16;
                ensure!(
                    g.id != 0 && value & 2 != 0,
                    "Нужны numbered variable Feature reports"
                );
                if let Some(old) = id_owner.insert(g.id, usage) {
                    ensure!(old == usage, "Несколько назначений у Report ID");
                }
                let entry = reports.entry(usage).or_insert((g.id, Vec::new()));
                ensure!(
                    entry.0 == g.id && !usages.is_empty(),
                    "Неоднозначный отчёт LampArray"
                );
                for i in 0..g.count as usize {
                    let u = *usages.get(i).unwrap_or_else(|| usages.last().unwrap());
                    ensure!(u >> 16 == 0x59, "Чужая UsagePage в LampArray");
                    entry.1.push(((u & 0xffff) as u16, g.bits));
                }
            }
            _ => {}
        }
        if tag & 0x0c == 0 {
            usages.clear();
            minimum = None;
        }
    }
    ensure!(
        collections.is_empty() && saved.is_empty(),
        "Незакрытый HID scope"
    );
    let expected = [
        (
            2,
            vec![(3, 16), (4, 32), (5, 32), (6, 32), (7, 32), (8, 32)],
        ),
        (0x20, vec![(0x21, 16)]),
        (
            0x22,
            vec![
                (0x21, 16),
                (0x23, 32),
                (0x24, 32),
                (0x25, 32),
                (0x27, 32),
                (0x26, 32),
                (0x28, 8),
                (0x29, 8),
                (0x2a, 8),
                (0x2b, 8),
                (0x2c, 8),
                (0x2d, 8),
            ],
        ),
        (
            0x60,
            vec![
                (0x55, 8),
                (0x61, 16),
                (0x62, 16),
                (0x51, 8),
                (0x52, 8),
                (0x53, 8),
                (0x54, 8),
            ],
        ),
        (0x70, vec![(0x71, 8)]),
    ];
    for (u, fields) in expected {
        ensure!(
            reports.get(&u).is_some_and(|(_, f)| *f == fields),
            "Нестандартный порядок или ширина полей отчёта {u:02X}"
        );
    }
    let multi = &reports.get(&0x50).context("Нет MultiUpdate")?.1;
    ensure!(
        multi.len() >= 7 && (multi.len() - 2).is_multiple_of(5),
        "Неверный MultiUpdate"
    );
    let capacity = (multi.len() - 2) / 5;
    ensure!(
        (1..=32).contains(&capacity),
        "Необычная ёмкость MultiUpdate"
    );
    let mut fields = vec![(3, 8), (0x55, 8)];
    fields.extend(vec![(0x21, 16); capacity]);
    for _ in 0..capacity {
        fields.extend([(0x51, 8), (0x52, 8), (0x53, 8), (0x54, 8)]);
    }
    ensure!(*multi == fields, "Нестандартный MultiUpdate");
    Ok(ReportMap {
        attributes: reports[&2].0,
        request: reports[&0x20].0,
        response: reports[&0x22].0,
        multi: reports[&0x50].0,
        range: reports[&0x60].0,
        control: reports[&0x70].0,
        capacity,
    })
}

pub fn multi_packet(map: &ReportMap, lamps: &[(u16, [u8; 4])], complete: bool) -> Result<Vec<u8>> {
    ensure!(
        !lamps.is_empty() && lamps.len() <= map.capacity,
        "Неверное число ламп"
    );
    let mut p = vec![0; 3 + 6 * map.capacity];
    p[0] = map.multi;
    p[1] = lamps.len() as u8;
    p[2] = u8::from(complete);
    for (i, (id, rgb)) in lamps.iter().enumerate() {
        p[3 + i * 2..5 + i * 2].copy_from_slice(&id.to_le_bytes());
        let at = 3 + map.capacity * 2 + i * 4;
        p[at..at + 4].copy_from_slice(rgb);
    }
    Ok(p)
}
pub fn level(value: u8, count: u8) -> u8 {
    match count {
        0 | 255 => value,
        1 => u8::from(value >= 128),
        n => ((value as u32 * (n - 1) as u32 + 127) / 255) as u8,
    }
}

#[cfg(windows)]
pub struct Device {
    pub name: String,
    pub identity: String,
    pub count: u16,
    pub interval_us: u32,
    device: hidapi::HidDevice,
    map: ReportMap,
    controlled: bool,
}
#[cfg(windows)]
impl Device {
    fn feature(device: &hidapi::HidDevice, id: u8, size: usize) -> Result<Vec<u8>> {
        let mut p = vec![0; size + 1];
        p[0] = id;
        let n = device.get_feature_report(&mut p)?;
        ensure!(
            n == p.len() && p[0] == id,
            "Обрезанный или чужой Feature Report"
        );
        Ok(p[1..].to_vec())
    }
    pub fn discover() -> Result<(Vec<Self>, Vec<String>)> {
        let api = hidapi::HidApi::new()?;
        let mut devices = Vec::new();
        let mut notes = Vec::new();
        for info in api
            .device_list()
            .filter(|d| d.usage_page() == 0x59 && d.usage() == 1)
        {
            let name = info.product_string().unwrap_or("LampArray").to_string();
            let result = (|| -> Result<Self> {
                let device = info.open_device(&api)?;
                let mut descriptor = [0; 16384];
                let n = device.get_report_descriptor(&mut descriptor)?;
                ensure!(n < descriptor.len(), "Дескриптор HID обрезан");
                let map = parse_descriptor(&descriptor[..n])?;
                let a = Self::feature(&device, map.attributes, 22)?;
                let count = u16::from_le_bytes(a[..2].try_into()?);
                let kind = u32::from_le_bytes(a[14..18].try_into()?);
                ensure!(kind == 1, "LampArray не является клавиатурой");
                ensure!((1..=512).contains(&count), "Необычное число ламп");
                let interval_us = u32::from_le_bytes(a[18..22].try_into()?);
                ensure!(interval_us <= 1_000_000, "Слишком медленное устройство");
                let identity = crate::firmware_assistant::digest(info.path().to_bytes());
                Ok(Self {
                    name: name.clone(),
                    identity,
                    count,
                    interval_us,
                    device,
                    map,
                    controlled: false,
                })
            })();
            match result {
                Ok(d) => devices.push(d),
                Err(e) => notes.push(format!("{name}: {e:#}")),
            }
        }
        Ok((devices, notes))
    }
    pub fn solid(&mut self, rgb: [u8; 3]) -> Result<String> {
        let _lease = crate::hardware::DeviceLease::acquire(&self.identity[..16])?;
        let mut lamps = Vec::new();
        // Selecting lamp metadata is a standard query; it does not change lighting or flash.
        for id in 0..self.count {
            let mut request = vec![self.map.request];
            request.extend_from_slice(&id.to_le_bytes());
            self.device.send_feature_report(&request)?;
            let a = Self::feature(&self.device, self.map.response, 28)?;
            ensure!(
                u16::from_le_bytes(a[..2].try_into()?) == id,
                "LampId не совпал"
            );
            if a[26] != 0 {
                lamps.push((
                    id,
                    [
                        level(rgb[0], a[22]),
                        level(rgb[1], a[23]),
                        level(rgb[2], a[24]),
                        level(255, a[25]),
                    ],
                ));
            }
        }
        ensure!(!lamps.is_empty(), "Нет программируемых светодиодов");
        self.device.send_feature_report(&[self.map.control, 0])?;
        self.controlled = true;
        let result = (|| -> Result<()> {
            let chunks = lamps.len().div_ceil(self.map.capacity);
            for (i, group) in lamps.chunks(self.map.capacity).enumerate() {
                self.device.send_feature_report(&multi_packet(
                    &self.map,
                    group,
                    i + 1 == chunks,
                )?)?;
                std::thread::sleep(std::time::Duration::from_micros(
                    self.interval_us.max(1000) as u64
                ));
            }
            Ok(())
        })();
        if let Err(e) = result {
            let _ = self.restore();
            return Err(e);
        }
        Ok(format!(
            "Цвет отправлен через HID LampArray: {} светодиодов. Физический результат проверьте на клавиатуре.",
            lamps.len()
        ))
    }
    pub fn restore(&mut self) -> Result<String> {
        if self.controlled {
            self.device.send_feature_report(&[self.map.control, 1])?;
            self.controlled = false;
        }
        Ok("Клавиатуре возвращено автономное управление подсветкой.".into())
    }
}
#[cfg(windows)]
impl Drop for Device {
    fn drop(&mut self) {
        if self.controlled {
            let _ = self.restore();
        }
    }
}
