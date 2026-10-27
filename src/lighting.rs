use anyhow::{Context, Result};
#[derive(Clone, Debug)]
pub struct Target {
    pub label: String,
    pub detail: String,
    pub writable: bool,
}
pub enum Session {
    OpenRgb(crate::openrgb::Client),
    #[cfg(windows)]
    LampArray(Vec<crate::lamparray::Device>, Vec<String>),
    #[cfg(windows)]
    Aa55(Vec<crate::aa55::Device>, Vec<String>),
    #[cfg(windows)]
    Feature(Vec<crate::sonix_feature::Device>, Vec<String>),
}
impl Session {
    pub fn connect(backend: u8) -> Result<Self> {
        if backend == 1 {
            return Ok(Self::OpenRgb(crate::openrgb::Client::connect(6742)?));
        }
        #[cfg(windows)]
        {
            if backend == 2 {
                let (devices, notes) = crate::aa55::Device::discover()?;
                return Ok(Self::Aa55(devices, notes));
            }
            if backend == 3 {
                let (devices, notes) = crate::sonix_feature::Device::discover()?;
                return Ok(Self::Feature(devices, notes));
            }
            anyhow::ensure!(backend == 0, "Неизвестный адаптер");
            let (devices, notes) = crate::lamparray::Device::discover()?;
            Ok(Self::LampArray(devices, notes))
        }
        #[cfg(not(windows))]
        {
            anyhow::bail!("USB LampArray доступен в Windows")
        }
    }
    pub fn targets(&self) -> Vec<Target> {
        match self {
            #[cfg(windows)]
            Self::Aa55(devices, _) => devices
                .iter()
                .map(|d| Target {
                    label: d.name.clone(),
                    detail: format!(
                        "Штатный AA55 · USB {:04X}:{:04X} · версия {}",
                        d.info.vid, d.info.pid, d.info.version
                    ),
                    writable: true,
                })
                .collect(),
            #[cfg(windows)]
            Self::Feature(devices, _) => devices
                .iter()
                .map(|d| Target {
                    label: d.name.clone(),
                    detail: format!(
                        "SONiX Feature · экспериментально · bcdDevice {:04X}",
                        d.release
                    ),
                    writable: true,
                })
                .collect(),
            Self::OpenRgb(s) => s
                .devices
                .iter()
                .map(|d| Target {
                    label: format!("{} · {}", d.vendor, d.name),
                    detail: format!(
                        "OpenRGB · {} LED · {}",
                        d.led_count,
                        if d.writable() {
                            "Direct"
                        } else {
                            "нет подходящего Direct"
                        }
                    ),
                    writable: d.writable(),
                })
                .collect(),
            #[cfg(windows)]
            Self::LampArray(devices, _) => devices
                .iter()
                .map(|d| Target {
                    label: d.name.clone(),
                    detail: format!("USB LampArray · {} LED", d.count),
                    writable: true,
                })
                .collect(),
        }
    }
    pub fn notes(&self) -> String {
        match self {
            Self::OpenRgb(s) => format!(
                "SDK protocol {}. Видны только клавиатуры. Резервная копия: %LOCALAPPDATA%\\AjazzConnect\\backups.",
                s.version
            ),
            #[cfg(windows)]
            Self::LampArray(_, notes) => notes.join("\n"),
            #[cfg(windows)]
            Self::Aa55(_, notes) => format!(
                "Сохраняемая настройка цвета; потоковая запись отключена. Резервная копия: %LOCALAPPDATA%\\AjazzConnect\\backups.\n{}",
                notes.join("\n")
            ),
            #[cfg(windows)]
            Self::Feature(_, notes) => format!(
                "Записывается настройка подсветки. В этом протоколе нет подтверждённой команды чтения исходного эффекта: резервная копия и автоматический откат недоступны. Аппаратная проверка этой реализации здесь не проводилась.\n{}",
                notes.join("\n")
            ),
        }
    }
    pub fn solid(&mut self, index: usize, rgb: [u8; 3]) -> Result<String> {
        match self {
            #[cfg(windows)]
            Self::Aa55(devices, _) => devices
                .get_mut(index)
                .context("Выберите клавиатуру")?
                .solid(rgb),
            #[cfg(windows)]
            Self::Feature(devices, _) => devices
                .get(index)
                .context("Выберите клавиатуру")?
                .apply(rgb),
            Self::OpenRgb(s) => s.solid(index, rgb),
            #[cfg(windows)]
            Self::LampArray(devices, _) => devices
                .get_mut(index)
                .context("Выберите клавиатуру")?
                .solid(rgb),
        }
    }
    pub fn restore(&mut self, index: usize) -> Result<String> {
        match self {
            #[cfg(windows)]
            Self::Aa55(devices, _) => devices.get(index).context("Выберите клавиатуру")?.restore(),
            #[cfg(windows)]
            Self::Feature(_, _) => anyhow::bail!(
                "Исходный эффект не считывается; выберите режим сочетанием Fn на клавиатуре"
            ),
            Self::OpenRgb(s) => s.restore(index),
            #[cfg(windows)]
            Self::LampArray(devices, _) => devices
                .get_mut(index)
                .context("Выберите клавиатуру")?
                .restore(),
        }
    }
    pub fn restore_file(&mut self, path: &std::path::Path) -> Result<String> {
        use std::io::Read;
        let mut bytes = Vec::new();
        std::fs::File::open(path)?
            .take(128 * 1024 + 1)
            .read_to_end(&mut bytes)?;
        anyhow::ensure!(bytes.len() <= 128 * 1024, "Резервная копия слишком велика");
        match self {
            Self::OpenRgb(s) => {
                let backup = serde_json::from_slice(&bytes)?;
                s.restore_backup(&backup)?;
                Ok("Состояние восстановлено из резервной копии OpenRGB.".into())
            }
            #[cfg(windows)]
            Self::LampArray(_, _) => anyhow::bail!(
                "LampArray возвращается в автономный режим кнопкой «Вернуть подсветку»"
            ),
            #[cfg(windows)]
            Self::Aa55(devices, _) => {
                let b: crate::aa55::Backup = serde_json::from_slice(&bytes)?;
                devices
                    .iter()
                    .find(|d| d.identity == b.device_identity)
                    .context("Устройство резервной копии не найдено")?
                    .restore_backup(&b)
            }
            #[cfg(windows)]
            Self::Feature(_, _) => {
                anyhow::bail!("Резервные копии эффекта SONiX Feature недоступны")
            }
        }
    }
    pub fn restorable(&self) -> bool {
        #[cfg(windows)]
        {
            !matches!(self, Self::Feature(_, _))
        }
        #[cfg(not(windows))]
        {
            true
        }
    }
    pub fn file_restore(&self) -> bool {
        match self {
            Self::OpenRgb(_) => true,
            #[cfg(windows)]
            Self::Aa55(_, _) => true,
            #[cfg(windows)]
            _ => false,
        }
    }
}
