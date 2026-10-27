//! GPWB2401 preflight and package inspection. No ISP/reset/erase/program path.
//! The retail marking has not been mapped to a verified hardware profile.
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, path::Path};

pub const TARGET_MARKING: &str = "GPWB2401";
pub const PROFILE_VERIFIED: bool = false;
const MAX_PACKAGE: u64 = 32 * 1024 * 1024;

pub fn digest(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Interface {
    pub id: String,
    pub vid: u16,
    pub pid: u16,
    pub product: String,
    pub usage_page: u16,
    pub usage: u16,
    pub interface: i32,
    pub input_bytes: Option<u16>,
    pub output_bytes: Option<u16>,
    pub feature_bytes: Option<u16>,
    pub info: Option<crate::protocol::KeyboardInfo>,
    pub info_payload_hex: Option<String>,
    pub read_error: Option<String>,
    #[serde(default)]
    pub protocol_family: String,
    #[serde(default)]
    pub control_hint: String,
    #[serde(default)]
    pub bcd_device: u16,
    #[serde(default)]
    pub report_descriptor_hex: Option<String>,
    #[serde(default)]
    pub report_descriptor_sha256: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Report {
    pub schema: u32,
    pub application_version: String,
    pub target_marking: String,
    pub target_profile_verified: bool,
    pub created_utc: String,
    pub interfaces: Vec<Interface>,
    pub firmware: Option<PackageInfo>,
    pub firmware_writes: bool,
    pub settings_writes: bool,
    pub boot_requests: bool,
    pub notice: String,
}

impl Report {
    pub fn empty() -> Self {
        Self {
            schema: 1,
            application_version: crate::version().into(),
            target_marking: TARGET_MARKING.into(),
            target_profile_verified: PROFILE_VERIFIED,
            created_utc: chrono::Utc::now().to_rfc3339(),
            interfaces: Vec::new(),
            firmware: None,
            firmware_writes: false,
            settings_writes: false,
            boot_requests: false,
            notice: "Для GPWB2401 ещё не подтверждены аппаратная ревизия и подходящий образ. Сохраните отчёт после проверки по USB.".into(),
        }
    }

    pub fn wired_info(&self) -> Option<&crate::protocol::KeyboardInfo> {
        self.interfaces.iter().find_map(|i| i.info.as_ref())
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        crate::storage::write_json_atomic(path, self)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImageInfo {
    pub bytes: usize,
    pub sha256: String,
    pub word_sum: Option<String>,
    pub initial_sp: u32,
    pub reset: u32,
    pub plausible_vectors: bool,
    pub vid: Option<u16>,
    pub pid: Option<u16>,
    pub version_raw: Option<u16>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PackageInfo {
    pub kind: String,
    pub sha256: String,
    pub bytes: usize,
    pub image: ImageInfo,
    pub metadata: BTreeMap<u32, String>,
    pub advertised_checksum_matches: Option<bool>,
    pub compatibility_proven: bool,
    pub vendor_code_executed: bool,
}

fn slice(data: &[u8], at: usize, size: usize) -> Result<&[u8]> {
    let end = at.checked_add(size).context("Переполнение смещения")?;
    data.get(at..end)
        .context("Повреждённый или обрезанный файл")
}
fn u16_at(data: &[u8], at: usize) -> Result<u16> {
    Ok(u16::from_le_bytes(slice(data, at, 2)?.try_into()?))
}
fn u32_at(data: &[u8], at: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(slice(data, at, 4)?.try_into()?))
}

fn image_info(data: &[u8]) -> Result<ImageInfo> {
    ensure!(
        (64..=1024 * 1024).contains(&data.len()),
        "Неизвестный размер образа"
    );
    let initial_sp = u32_at(data, 0)?;
    let reset = u32_at(data, 4)?;
    let has_identity = data.get(0x9000..0x9004) == Some(&[0, 0, 0, 0x12]);
    let word_sum = data.len().is_multiple_of(2).then(|| {
        format!(
            "{:04X}",
            data.as_chunks::<2>().0.iter().fold(0u16, |s, p| s
                .wrapping_add(u16::from_le_bytes([p[0], p[1]])))
        )
    });
    Ok(ImageInfo {
        bytes: data.len(),
        sha256: digest(data),
        word_sum,
        initial_sp,
        reset,
        plausible_vectors: (0x20000000..0x20080000).contains(&initial_sp)
            && reset & 1 == 1
            && (reset as usize) < data.len(),
        vid: if has_identity {
            Some(u16_at(data, 0x9004)?)
        } else {
            None
        },
        pid: if has_identity {
            Some(u16_at(data, 0x9006)?)
        } else {
            None
        },
        version_raw: if has_identity {
            Some(u16_at(data, 0x9008)?)
        } else {
            None
        },
    })
}

struct Pe<'a> {
    data: &'a [u8],
    sections: Vec<(usize, usize, usize)>,
    tree: &'a [u8],
}
impl<'a> Pe<'a> {
    fn new(data: &'a [u8]) -> Result<Self> {
        ensure!(data.starts_with(b"MZ"), "Ожидается PE EXE");
        let nt = u32_at(data, 0x3c)? as usize;
        ensure!(
            slice(data, nt, 4)? == b"PE\0\0",
            "Повреждённый заголовок PE"
        );
        let count = u16_at(data, nt + 6)? as usize;
        ensure!((1..=96).contains(&count), "Некорректная таблица PE");
        let size = u16_at(data, nt + 20)? as usize;
        let opt = nt + 24;
        slice(data, opt, size)?;
        let dd = match u16_at(data, opt)? {
            0x10b => 96,
            0x20b => 112,
            _ => bail!("Неизвестный формат PE"),
        };
        ensure!(size >= dd + 24, "Нет таблицы ресурсов");
        let mut sections = Vec::new();
        for i in 0..count {
            let row = slice(data, opt + size + i * 40, 40)?;
            let rva = u32_at(row, 12)? as usize;
            let bytes = u32_at(row, 16)? as usize;
            let offset = u32_at(row, 20)? as usize;
            slice(data, offset, bytes)?;
            sections.push((rva, bytes, offset));
        }
        let mut pe = Self {
            data,
            sections,
            tree: &[],
        };
        let rva = u32_at(data, opt + dd + 16)? as usize;
        let bytes = u32_at(data, opt + dd + 20)? as usize;
        ensure!(bytes > 0, "В EXE нет ресурсов");
        pe.tree = pe.rva(rva, bytes)?;
        Ok(pe)
    }
    fn rva(&self, address: usize, bytes: usize) -> Result<&'a [u8]> {
        let mut matches = Vec::new();
        for &(start, size, offset) in &self.sections {
            if let Some(delta) = address.checked_sub(start)
                && delta <= size
                && bytes <= size - delta
            {
                matches.push(slice(self.data, offset + delta, bytes)?);
            }
        }
        ensure!(
            matches.len() == 1,
            "Некорректный или неоднозначный адрес ресурса"
        );
        Ok(matches[0])
    }
    fn entries(&self, at: usize) -> Result<Vec<(u32, u32)>> {
        let count = u16_at(self.tree, at + 12)? as usize + u16_at(self.tree, at + 14)? as usize;
        ensure!(count <= 4096, "Слишком большая таблица ресурсов");
        (0..count)
            .map(|i| {
                let row = slice(self.tree, at + 16 + i * 8, 8)?;
                Ok((u32_at(row, 0)?, u32_at(row, 4)?))
            })
            .collect()
    }
    fn resources(&self) -> Result<BTreeMap<u32, &'a [u8]>> {
        let roots: Vec<_> = self
            .entries(0)?
            .into_iter()
            .filter(|(id, _)| *id == 10)
            .collect();
        ensure!(
            roots.len() == 1 && roots[0].1 & 0x80000000 != 0,
            "Нет однозначного RCDATA"
        );
        let mut result = BTreeMap::new();
        for (id, directory) in self.entries((roots[0].1 & 0x7fffffff) as usize)? {
            ensure!(
                id & 0x80000000 == 0 && directory & 0x80000000 != 0,
                "Неизвестное дерево ресурсов"
            );
            let language = self.entries((directory & 0x7fffffff) as usize)?;
            ensure!(
                language.len() == 1 && language[0].1 & 0x80000000 == 0,
                "Неоднозначный язык ресурса"
            );
            let item = slice(self.tree, language[0].1 as usize, 16)?;
            let value = self.rva(u32_at(item, 0)? as usize, u32_at(item, 4)? as usize)?;
            ensure!(result.insert(id, value).is_none(), "Повторный RCDATA");
        }
        Ok(result)
    }
}

pub fn inspect_bytes(data: &[u8]) -> Result<PackageInfo> {
    ensure!(data.len() as u64 <= MAX_PACKAGE, "Файл превышает 32 МБ");
    let mut metadata = BTreeMap::new();
    let image;
    let kind = if data.starts_with(b"MZ") {
        let pe = Pe::new(data)?;
        let resources = pe.resources()?;
        image = image_info(
            resources
                .get(&4000)
                .context("В этом пакете не найден образ RCDATA4000")?,
        )?;
        for id in [4007, 4008, 4009, 4010, 4013, 4014, 4016] {
            if let Some(bytes) = resources.get(&id) {
                ensure!(
                    bytes.len() <= 1024 && bytes.len().is_multiple_of(2),
                    "Некорректная строка метаданных"
                );
                let words: Vec<_> = bytes
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|x| u16::from_le_bytes([x[0], x[1]]))
                    .collect();
                metadata.insert(
                    id,
                    String::from_utf16(&words)?
                        .trim_end_matches('\0')
                        .to_owned(),
                );
            }
        }
        "SONiX PE / RCDATA4000"
    } else {
        ensure!(
            !data.starts_with(b":") && !data.starts_with(b"<") && !data.starts_with(b"PK"),
            "Выберите SONiX EXE или исходный BIN; HTML, HEX и ZIP здесь не интерпретируются как BIN"
        );
        image = image_info(data)?;
        ensure!(image.plausible_vectors, "Файл не распознан как ARM-образ");
        "ARM BIN"
    };
    let advertised_checksum_matches = metadata
        .get(&4014)
        .map(|sum| image.word_sum.as_deref() == Some(sum.to_uppercase().as_str()));
    Ok(PackageInfo {
        kind: kind.into(),
        sha256: digest(data),
        bytes: data.len(),
        image,
        metadata,
        advertised_checksum_matches,
        compatibility_proven: false,
        vendor_code_executed: false,
    })
}

pub fn inspect_file(path: &Path) -> Result<PackageInfo> {
    use std::io::Read;
    let file = std::fs::File::open(path)?;
    let mut data = Vec::new();
    file.take(MAX_PACKAGE + 1).read_to_end(&mut data)?;
    inspect_bytes(&data)
}

#[cfg(windows)]
mod win {
    use super::*;
    use std::{
        ffi::c_void,
        time::{Duration, Instant},
    };

    #[repr(C)]
    #[derive(Default)]
    struct Caps {
        usage: u16,
        usage_page: u16,
        input: u16,
        output: u16,
        feature: u16,
        reserved: [u16; 17],
        counts: [u16; 10],
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn CreateFileW(
            path: *const u16,
            access: u32,
            share: u32,
            security: *const c_void,
            creation: u32,
            flags: u32,
            template: *mut c_void,
        ) -> *mut c_void;
        fn CloseHandle(handle: *mut c_void) -> i32;
    }
    #[link(name = "hid", kind = "raw-dylib")]
    unsafe extern "system" {
        fn HidD_GetPreparsedData(handle: *mut c_void, data: *mut *mut c_void) -> u8;
        fn HidD_FreePreparsedData(data: *mut c_void) -> u8;
        fn HidP_GetCaps(data: *mut c_void, caps: *mut Caps) -> i32;
    }
    fn caps(path: &str) -> Result<Caps> {
        let wide: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
        let handle = unsafe {
            CreateFileW(
                wide.as_ptr(),
                0,
                3,
                std::ptr::null(),
                3,
                0,
                std::ptr::null_mut(),
            )
        };
        ensure!(
            handle as isize != -1 && !handle.is_null(),
            "Не удалось прочитать HID-дескриптор"
        );
        let mut data = std::ptr::null_mut();
        let ok = unsafe { HidD_GetPreparsedData(handle, &mut data) } != 0;
        let mut caps = Caps::default();
        let status = if ok {
            unsafe { HidP_GetCaps(data, &mut caps) }
        } else {
            0
        };
        unsafe {
            if !data.is_null() {
                HidD_FreePreparsedData(data);
            }
            CloseHandle(handle);
        }
        ensure!(status == 0x110000, "HID capabilities недоступны");
        Ok(caps)
    }
    pub(super) fn shape(path: &str) -> Result<(u16, u16, u16)> {
        let c = caps(path)?;
        Ok((c.input, c.output, c.feature))
    }
    fn read_info(device: &hidapi::HidDevice) -> Result<Vec<u8>> {
        // This is the sole command in the diagnostic probe. Lighting adapters
        // are separate and require the user's explicit Apply action.
        let mut drained = false;
        for _ in 0..32 {
            let mut packet = [0; 65];
            if device.read_timeout(&mut packet, 1)? == 0 {
                drained = true;
                break;
            }
        }
        ensure!(drained, "Закройте другие драйверы: поток HID занят");
        let mut request = [0; 65];
        request[1..9].copy_from_slice(&[0xaa, 0x10, 48, 0, 0, 0, 1, 0]);
        let n = device.write(&request)?;
        ensure!(
            n == 0 || n == 65,
            "Не удалось отправить запрос идентификации"
        );
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            let mut raw = [0; 65];
            let n = device.read_timeout(&mut raw, 100)?;
            let data = if n == 65 && raw[0] == 0 {
                &raw[1..n]
            } else {
                &raw[..n]
            };
            if data.len() == 64 && data[..5] == [0x55, 0x10, 48, 0, 0] {
                return Ok(data[8..56].to_vec());
            }
        }
        bail!("Клавиатура не ответила на GET_INFO; других команд не отправлено")
    }
    pub fn probe() -> Result<Report> {
        let api = hidapi::HidApi::new()?;
        let mut report = Report::empty();
        let endpoints: Vec<_> = api
            .device_list()
            .filter(|d| {
                [0x0c45, 0x0416, 0x3151, 0x258a, 0x320f, 0x3554].contains(&d.vendor_id())
                    || (d.usage_page() == 0x59 && d.usage() == 1)
                    || (d.usage_page() == 0xff60 && d.usage() == 0x61)
                    || d.product_string().is_some_and(|s| {
                        let name = s.to_ascii_uppercase();
                        name.contains("AJAZZ") || name.contains("AK820")
                    })
            })
            .collect();
        let _lease = crate::hardware::DeviceLease::acquire("AK820MAX").ok();
        for d in endpoints {
            let path = d.path().to_string_lossy();
            let caps = caps(&path);
            let mut interface = Interface {
                id: digest(path.as_bytes())[..16].into(),
                vid: d.vendor_id(),
                pid: d.product_id(),
                product: d.product_string().unwrap_or("").into(),
                usage_page: d.usage_page(),
                usage: d.usage(),
                interface: d.interface_number(),
                input_bytes: caps.as_ref().ok().map(|c| c.input),
                output_bytes: caps.as_ref().ok().map(|c| c.output),
                feature_bytes: caps.as_ref().ok().map(|c| c.feature),
                info: None,
                info_payload_hex: None,
                read_error: caps.as_ref().err().map(|e| format!("{e:#}")),
                protocol_family: String::new(),
                control_hint: String::new(),
                bcd_device: d.release_number(),
                report_descriptor_hex: None,
                report_descriptor_sha256: None,
            };
            // The Windows native backend reconstructs this from HID preparsed
            // data. No Output/Feature request or keyboard input read is made.
            if (d.usage_page() >= 0xff00 || d.usage_page() == 0x59)
                && let Ok(device) = d.open_device(&api)
            {
                let mut bytes = [0u8; 16384];
                if let Ok(n) = device.get_report_descriptor(&mut bytes)
                    && n < bytes.len()
                {
                    interface.report_descriptor_hex =
                        Some(bytes[..n].iter().map(|b| format!("{b:02x}")).collect());
                    interface.report_descriptor_sha256 = Some(digest(&bytes[..n]));
                }
            }
            if d.vendor_id() == 0x0c45
                && [0x80a0, 0x80be, 0x8033, 0x8009, 0x800a].contains(&d.product_id())
                && d.usage_page() == 0xff68
                && d.usage() == 97
                && caps.as_ref().is_ok_and(|c| c.input == 65 && c.output == 65)
            {
                let result = if _lease.is_some() {
                    d.open_device(&api)
                        .map_err(anyhow::Error::from)
                        .and_then(|device| read_info(&device))
                } else {
                    Err(anyhow::anyhow!(
                        "Клавиатура занята AJAZZ Studio; закройте её и повторите проверку"
                    ))
                };
                match result {
                    Ok(bytes) => {
                        interface.info_payload_hex =
                            Some(bytes.iter().map(|b| format!("{b:02x}")).collect());
                        match crate::protocol::KeyboardInfo::parse(&bytes) {
                            Ok(info) => interface.info = Some(info),
                            Err(e) => interface.read_error = Some(format!("{e:#}")),
                        }
                    }
                    Err(e) => interface.read_error = Some(format!("{e:#}")),
                }
            }
            let (family, hint) = crate::families::classify(&interface);
            interface.protocol_family = family.into();
            interface.control_hint = hint.into();
            report.interfaces.push(interface);
        }
        Ok(report)
    }
}

pub fn probe() -> Result<Report> {
    #[cfg(windows)]
    {
        win::probe()
    }
    #[cfg(not(windows))]
    {
        bail!("Определение клавиатуры доступно в Windows")
    }
}

#[cfg(windows)]
pub(crate) fn endpoint_shape(path: &str) -> Result<(u16, u16, u16)> {
    win::shape(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn marking_never_enables_flash() {
        let report = Report::empty();
        assert!(!report.target_profile_verified);
        assert!(!report.firmware_writes && !report.settings_writes && !report.boot_requests);
        assert_eq!(report.target_marking, "GPWB2401");
    }
    #[test]
    fn report_contains_no_serial_or_device_path() {
        let json = serde_json::to_string(&Report::empty()).unwrap();
        assert!(!json.contains("serial") && !json.contains("device_path"));
    }
    #[test]
    fn malformed_inputs_do_not_become_images() {
        for data in [
            b"MZ".as_slice(),
            b"<!DOCTYPE html>",
            b"PK\x03\x04",
            b":00000001FF",
        ] {
            assert!(inspect_bytes(data).is_err());
        }
        assert!(inspect_bytes(&[0; 64]).is_err());
    }
    #[test]
    fn raw_image_has_no_automatic_compatibility() {
        let mut data = vec![0; 64];
        data[..4].copy_from_slice(&0x20001000u32.to_le_bytes());
        data[4..8].copy_from_slice(&9u32.to_le_bytes());
        let parsed = inspect_bytes(&data).unwrap();
        assert!(parsed.image.plausible_vectors);
        assert!(!parsed.compatibility_proven && !parsed.vendor_code_executed);
    }
    #[test]
    fn parses_original_active_resource_not_template() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sonix-136.exe");
        if !path.exists() {
            return;
        }
        let report = inspect_file(&path).unwrap();
        assert_eq!(
            report.image.sha256,
            "58db8a73ea8be7b8f730d67177f3ad78ec1b28b2b73ac15861e67963a1b116ff"
        );
        assert_eq!(report.image.pid, Some(0x80a0));
        assert_eq!(report.advertised_checksum_matches, Some(true));
        assert_eq!(report.metadata.get(&4010).unwrap(), "SN34F280");
    }
    #[test]
    fn inspection_rejects_truncated_resources() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sonix-136.exe");
        if !path.exists() {
            return;
        }
        let data = std::fs::read(path).unwrap();
        for n in [64, 512, 4096, data.len() / 2] {
            assert!(inspect_bytes(&data[..n]).is_err());
        }
    }
}
