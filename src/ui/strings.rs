//! Every user-facing string. Russian is the current default; translating the interface means
//! translating this file. Error texts name the cause and the action (`docs/DESIGN.md`, section 10).

use crate::core::{Detail, DeviceKind, DriverError, Maturity, Outcome, PowerSource, Requirement};

pub const NAV_DEVICES: &str = "Устройства";
pub const NAV_DIAGNOSTICS: &str = "Диагностика";
pub const NAV_SETTINGS: &str = "Настройки";
pub const NAV_TOGGLE: &str = "Свернуть или развернуть меню";
pub const THEME_TO_LIGHT: &str = "Светлая тема";
pub const THEME_TO_DARK: &str = "Тёмная тема";
pub const BACK: &str = "Назад";

pub const DEVICES_TITLE: &str = "Устройства";
pub const DEVICES_LEAD: &str = "Подключённые устройства находятся сами, когда вы их подключаете. Поиск только читает данные и ничего не записывает.";
pub const DEVICES_FIND: &str = "Обновить";
pub const DEVICES_FIND_HINT: &str = "Искать устройства заново";
pub const DEVICES_FINDING: &str = "Поиск…";
pub const DEVICES_NEVER_SEARCHED: &str = "Ищем подключённые устройства…";
pub const DEVICES_NONE: &str = "Подходящие устройства не найдены.";
pub const DEVICES_NONE_HINT: &str =
    "Диагностика определит семейство контроллера и подготовит отчёт для добавления поддержки.";
pub const DEVICES_OPEN_DIAGNOSTICS: &str = "Открыть диагностику";
pub const DEVICES_NOTES: &str = "Подробности поиска";
pub const DEVICE_GONE: &str = "Устройство отключено.";
pub const DEVICE_DORMANT: &str = "Драйвер выключен";
pub const DEVICE_DORMANT_HINT: &str = "Устройство распознано, но его драйвер экспериментальный и выключен, поэтому приложение с ним не обменивается. Включите экспериментальные драйверы в настройках, чтобы управлять подсветкой.";
pub const DEVICE_OPEN_SETTINGS: &str = "Открыть настройки";
pub const EXPERIMENTAL: &str = "Экспериментально";

pub const LIGHTING: &str = "Подсветка";
pub const LIGHTING_APPLY: &str = "Применить цвет";
pub const LIGHTING_APPLY_PROGRAM: &str = "Применить";
pub const LIGHTING_RESTORE: &str = "Вернуть подсветку";
pub const LIGHTING_FROM_BACKUP: &str = "Из резервной копии…";
pub const LIGHTING_BACKUP_FILTER: &str = "Резервная копия подсветки";
pub const LIGHTING_COLOR: &str = "Цвет";
pub const LIGHTING_RUNNING: &str = "Обмен с устройством…";
pub const LIGHTING_PERSISTENT: &str = "Подсветка сохраняется в памяти устройства.";
pub const LIGHTING_BACKUP_FIRST: &str =
    "Перед первой записью исходное состояние сохраняется в резервную копию.";
pub const LIGHTING_NO_ROLLBACK: &str = "Отката нет: этот протокол не считывает исходный эффект. Вернуть режим можно сочетанием клавиш на устройстве.";
pub const LIGHTING_SESSION_ONLY: &str = "Цвет не сохраняется в устройстве: при закрытии приложения подсветка вернётся к собственной программе.";
pub const LIGHTING_NOT_WRITABLE: &str =
    "Устройство не предоставляет режим прямого управления цветом без автосохранения.";
pub const LIGHTING_EXPERIMENTAL: &str = "Драйвер не проверен на реальном устройстве. Применяйте, только если готовы проверить результат.";

pub const DIAG_TITLE: &str = "Диагностика";
pub const DIAG_LEAD: &str =
    "Пассивная проверка: читаются только дескрипторы и ответы на запросы идентификации.";
pub const DIAG_CONNECTION: &str = "Подключение";
pub const DIAG_CONNECTION_HINT: &str =
    "Подключите устройство кабелем и закройте фирменное ПО: оно может занимать канал.";
pub const DIAG_CHECK: &str = "Проверить подключение";
pub const DIAG_INTERFACES: &str = "Интерфейсы и семейства";
pub const DIAG_NOTHING: &str =
    "Известные интерфейсы не найдены. Проверьте кабель и режим подключения.";
pub const DIAG_REPORT: &str = "Отчёт";
pub const DIAG_REPORT_HINT: &str = "Отчёт помогает подготовить профиль устройства. В него не входят серийные номера, пути устройств, нажатия клавиш и личные файлы.";
pub const DIAG_SAVE: &str = "Сохранить отчёт…";
pub const DIAG_REPORT_FILTER: &str = "Отчёт";

pub const SETTINGS_TITLE: &str = "Настройки";
pub const SETTINGS_DRIVERS: &str = "Драйверы";
pub const SETTINGS_EXPERIMENTAL: &str = "Экспериментальные драйверы";
pub const SETTINGS_EXPERIMENTAL_HINT: &str = "Включает адаптеры, написанные по описаниям и не проверенные на устройствах. Они могут записывать настройки в память устройства.";
pub const SETTINGS_SAVE_FAILED: &str =
    "Не удалось сохранить настройки. Проверьте доступ к папке данных.";
pub const SETTINGS_DATA: &str = "Данные";
pub const SETTINGS_BACKUPS: &str = "Резервные копии подсветки";
pub const SETTINGS_OPEN_FOLDER: &str = "Открыть папку";
pub const SETTINGS_NO_DATA_DIR: &str =
    "Папка данных недоступна: не задана переменная LOCALAPPDATA.";
pub const SETTINGS_ABOUT: &str = "О приложении";
pub const SETTINGS_ABOUT_TEXT: &str = "Настройка периферии разных производителей через драйверы протоколов. Прошивка, загрузчик и сброс устройств не выполняются.";
pub const SETTINGS_ICONS: &str = "Иконки: Hugeicons (MIT).";
pub const SETTINGS_ANIMATIONS_OFF: &str = "Эффекты анимации Windows выключены: интерфейс использует только короткие переходы прозрачности.";

pub fn version(version: &str) -> String {
    format!("Версия {version}")
}

pub fn found(count: usize) -> String {
    format!("Найдено устройств: {count}")
}

pub fn interfaces(count: usize) -> String {
    format!("Интерфейсов: {count}")
}

pub fn saved_to(path: &str) -> String {
    format!("Сохранено: {path}")
}

pub fn save_failed(detail: &str) -> String {
    format!("Не удалось сохранить файл: {detail}. Выберите другую папку.")
}

pub fn kind(kind: DeviceKind) -> &'static str {
    match kind {
        DeviceKind::Keyboard => "Клавиатура",
        DeviceKind::Mouse => "Мышь",
        DeviceKind::Headset => "Гарнитура",
        DeviceKind::Gamepad => "Геймпад",
        DeviceKind::Lighting => "Подсветка",
        DeviceKind::Component => "Компонент ПК",
        DeviceKind::Other => "Устройство",
    }
}

pub fn maturity(maturity: Maturity) -> &'static str {
    match maturity {
        Maturity::Stable => "Основной",
        Maturity::Experimental => EXPERIMENTAL,
    }
}

pub const LABEL_DRIVER: &str = "Драйвер";
pub const LABEL_FIRMWARE: &str = "Прошивка";

/// (label, value) of one device fact.
pub fn detail(detail: &Detail) -> (&'static str, String) {
    match detail {
        Detail::Usb { vid, pid } => ("USB", format!("{vid:04X}:{pid:04X}")),
        Detail::Firmware(version) => (LABEL_FIRMWARE, version.clone()),
        Detail::Release(release) => ("bcdDevice", format!("{release:04X}")),
        Detail::Battery(percent) => (LABEL_BATTERY, battery(*percent)),
        Detail::Power(source) => (LABEL_POWER, power(*source).into()),
        Detail::Leds(count) => ("Светодиоды", count.to_string()),
    }
}

pub fn requirement(requirement: Requirement) -> String {
    match requirement {
        Requirement::LocalService { name, address } => {
            format!("Нужен запущенный {name} с включённым SDK-сервером на {address}.")
        }
    }
}

pub fn identified(firmware: Option<&str>) -> String {
    match firmware {
        Some(version) => {
            format!("Устройство ответило на запрос идентификации · прошивка {version}")
        }
        None => "Устройство ответило на запрос идентификации".into(),
    }
}

pub fn error(error: &DriverError) -> &'static str {
    match error {
        DriverError::Unavailable(_) => {
            "Служба или доступ к устройствам недоступны. Запустите нужную программу и повторите поиск."
        }
        DriverError::Busy => {
            "Устройство занято другой программой. Закройте фирменное ПО и повторите."
        }
        DriverError::Rejected(_) => {
            "Устройство не подошло драйверу: протокол не подтверждён. Ничего не записано."
        }
        DriverError::Protocol(_) => {
            "Ответ устройства не совпал с протоколом. Повторная запись не выполнялась."
        }
        DriverError::Unconfirmed(_) => {
            "Устройство не подтвердило команду. Повторная запись не выполнялась: проверьте результат на устройстве."
        }
        DriverError::ReadBackMismatch => {
            "Команда принята, но обратное чтение не совпало. Резервная копия сохранена, повторная запись не выполнялась."
        }
        DriverError::Changed => "Устройство или его прошивка изменились. Выполните поиск заново.",
        DriverError::NoRollback => {
            "В этом подключении подсветка ещё не менялась: возвращать нечего."
        }
        DriverError::ForeignBackup => {
            "Резервная копия относится к другому устройству или повреждена."
        }
        DriverError::Io(_) => {
            "Ошибка обмена с устройством. Переподключите его и выполните поиск заново."
        }
    }
}

pub fn outcome(outcome: Outcome) -> String {
    match outcome {
        Outcome::Confirmed => "Цвет применён и подтверждён обратным чтением.".into(),
        Outcome::ModeConfirmed => "Режим подтверждён; эта прошивка сообщает стандартную палитру. Проверьте цвет на устройстве.".into(),
        Outcome::Acknowledged => "Устройство подтвердило команды. Проверьте цвет на устройстве.".into(),
        Outcome::Sent { lamps } => format!("Цвет отправлен, светодиодов: {lamps}. Проверьте результат на устройстве."),
        Outcome::ServiceConfirmed => "Служба подтвердила новый цвет. Проверьте результат на устройстве.".into(),
        Outcome::Restored => "Исходная подсветка восстановлена.".into(),
        Outcome::Released => "Устройству возвращено собственное управление подсветкой.".into(),
    }
}
pub const SETTINGS_DEVELOPER: &str = "Режим разработчика";
pub const SETTINGS_ALL_DEVICES: &str = "Показывать все поддерживаемые устройства";
pub const SETTINGS_DEVELOPER_HINT: &str =
    "Каталог устройств для проверки дизайна без подключения оборудования.";
pub const DEVICE_CATALOG: &str = "Поддерживается · Предпросмотр";
pub const DEVICE_MULTIPLE_DRIVERS: &str = "Несколько драйверов";
pub const DEVICE_PREVIEW_DRIVER: &str = "Драйвер для предпросмотра";
pub const DEVICE_PREVIEW_VARIANTS: &str = "Предпросмотр вариантов драйвера";

pub const DEVICE_CONNECTED: &str = "Подключено";
pub const DEVICE_BATTERY_UNKNOWN: &str = "Заряд недоступен";

pub const LABEL_BATTERY: &str = "Заряд";
pub const LABEL_POWER: &str = "Питание";
pub fn power(source: PowerSource) -> &'static str {
    match source {
        PowerSource::Battery => "От батареи · не заряжается",
        PowerSource::External => "Внешнее питание",
        PowerSource::Unknown => "Статус питания неизвестен",
    }
}
pub fn battery(percent: u8) -> String {
    format!("{percent}%")
}

pub const DEVICE_NO_DRIVER: &str = "Не добавлен";
pub const DEVICE_PLACEHOLDER: &str = "Заглушка · предпросмотр";

pub fn battery_status(percent: Option<u8>, source: Option<PowerSource>) -> String {
    let Some(percent) = percent else {
        return DEVICE_BATTERY_UNKNOWN.into();
    };
    let state = match source {
        Some(PowerSource::External) if percent == 100 => "Заряжено",
        Some(PowerSource::External) => "Заряжается",
        Some(PowerSource::Battery) => "Не заряжается",
        _ => "",
    };
    if state.is_empty() {
        format!("{percent}%")
    } else {
        format!("{percent}% · {state}")
    }
}

pub const SETTINGS_BATTERY: &str = "Заряд устройств";
pub fn battery_mode_name(mode: crate::app::battery::BatteryMode) -> &'static str {
    use crate::app::battery::BatteryMode;
    match mode {
        BatteryMode::Automatic => "Автоматически",
        BatteryMode::Responsive => "Быстро",
        BatteryMode::Eco => "Экономно",
    }
}
pub fn battery_mode_hint(mode: crate::app::battery::BatteryMode) -> &'static str {
    use crate::app::battery::BatteryMode;
    match mode {
        BatteryMode::Automatic => "Карточка: 5 с. В фоне: 5 мин от батареи, 1 мин при зарядке.",
        BatteryMode::Responsive => "Карточка: 2 с. В фоне: 2 мин от батареи, 30 с при зарядке.",
        BatteryMode::Eco => "Карточка: 15 с. В фоне: 15 мин от батареи, 5 мин при зарядке.",
    }
}
pub const STUDIO_LIGHTING: &str = "Настроить подсветку";
pub const STUDIO_ENCODER: &str = "Настроить энкодер";
pub const STUDIO_PREVIEW: &str = "Предпросмотр · без записи в устройство";
pub const STUDIO_KEYS: &str = "Нажимайте клавиши в этом окне — модель откликнется";
pub const STUDIO_SETTINGS: &str = "Настройки";
pub const STUDIO_CLOSE_SETTINGS: &str = "Скрыть настройки";
pub const STUDIO_CLOSE_EDITOR: &str = "Закрыть редактор";
pub const STUDIO_HUE_HINT: &str = "Каждое кольцо — отдельный цвет подсветки";
pub const STUDIO_SOLID: &str = "Постоянный";
pub const STUDIO_WAVE: &str = "Волна";
pub const STUDIO_REACTIVE: &str = "Отклик";
pub const STUDIO_VOLUME: &str = "Громкость";
pub const STUDIO_SCROLL: &str = "Прокрутка";
pub const STUDIO_BRIGHTNESS: &str = "Яркость";
pub const STUDIO_REPLAY: &str = "Демонстрация";
pub const STUDIO_ENCODER_HINT: &str = "Назначение энкодера пока доступно только в предпросмотре.";
pub const STUDIO_ENCODER_STEP: &str = "Шаг поворота";
pub const STUDIO_ENCODER_PRESS: &str = "Нажатие";
pub const STUDIO_ENCODER_MUTE: &str = "Выключить звук";
pub const STUDIO_ENCODER_PAUSE: &str = "Пауза / продолжить";
pub const STUDIO_ENCODER_DEFAULT: &str = "Стандартное действие";
pub const STUDIO_DEVICE_WRITE: &str = "Применение к устройству";
pub const STUDIO_EFFECT_PREVIEW: &str = "Этот эффект пока доступен только на модели.";
pub const STUDIO_MORE: &str = "Сведения";
pub const LIGHTING_SPEED: &str = "Скорость";
pub const LIGHTING_DRAFT_HINT: &str = "Изменения отправляются кнопкой «Применить».";
pub const LIGHTING_NATIVE_PREVIEW: &str = "Условный предпросмотр. Эффект исполняется клавиатурой.";
pub const LIGHTING_CONNECT_USB: &str = "Для управления подсветкой подключите клавиатуру USB-кабелем и включите проводной режим. Сейчас доступен только предпросмотр.";
pub const LIGHTING_OFFLINE_CONTROLS: &str = "Предпросмотр настроек драйвера. Для записи откройте подключённое устройство; значения здесь примерные.";
pub const LIGHTING_PREVIEW_VALUES: &str =
    "Примерные настройки драйвера; состояние подсветки клавиатуры не считано.";
pub const LIGHTING_PREVIEW_ONLY: &str = "Изменения видны только на модели.";

pub fn lighting_effect(effect: crate::core::LightingEffect) -> &'static str {
    use crate::core::LightingEffect as E;
    match effect {
        E::Solid => "Постоянный",
        E::Rainbow => "Радуга",
        E::Breathing => "Дыхание",
        E::Flash => "Вспышки",
        E::Rain => "Капли дождя",
        E::RainbowWheel => "Радужное колесо",
        E::Ripples => "Круги",
        E::Stars => "Мерцание звёзд",
        E::Fade => "Затухание",
        E::Snake => "Змейка",
        E::Neon => "Неоновый поток",
        E::Reactive => "Отклик",
        E::Sine => "Синусоида",
        E::Scan => "Сканирование",
        E::Windmill => "Ветряная мельница",
        E::Waterfall => "Цветной водопад",
        E::Blossom => "Цветение",
        E::Storm => "Вихрь",
        E::Off => "Выключено",
    }
}
