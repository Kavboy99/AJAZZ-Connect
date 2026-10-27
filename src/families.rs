//! Passive classification. A family match is not a firmware compatibility proof.
use crate::firmware_assistant::Interface;
pub fn classify(i: &Interface) -> (&'static str, &'static str) {
    match (i.vid, i.pid, i.usage_page, i.usage) {
        (0x258a, 0x010c, _, _) => (
            "BY Tech / Sinowealth 010C candidate",
            "В пакете AK820MAX V1.0 найден этот USB-ID. Отдельный протокол Report ID 6; SONiX-команды не отправляются",
        ),
        (0x3554, 0xfa09, _, _) => (
            "BY / Compx RF receiver candidate",
            "Этот ID указан как беспроводной в AK820MAX V1.0. Проводной протокол к радиоканалу не применяется",
        ),
        (_, _, 0x59, 1) => (
            "HID LampArray",
            "Стандартный RGB: отдельное подключение на вкладке «Подсветка»; требуется проверка полей дескриптора",
        ),
        (0x0c45, 0x7210, _, _) => (
            "SONiX SN34F280 ROM ISP",
            "Загрузчик: команды подсветки и автоматическая запись запрещены",
        ),
        (0x0c45, 0xfefc, _, _) => (
            "AJAZZ RF receiver",
            "Радиоприёмник: его прошивка отличается от прошивки клавиатуры",
        ),
        (0x0c45, _, 0xff13, 1) if i.feature_bytes == Some(65) => (
            "SONiX Feature / candidate",
            "Семейство AK820 Pro/MAX/PLUS: настройки 04 xx; отдельный экспериментальный адаптер проверяет USB-контейнер и START",
        ),
        (0x0c45, _, 0xff68, 0x61) if i.output_bytes == Some(4097) => (
            "SONiX Feature data / candidate",
            "Большой канал данных. Не путать с каналом AA55 на 65 байт",
        ),
        (0x0c45, _, 0xff68, 0x61) if i.output_bytes == Some(65) && i.input_bytes == Some(65) => (
            "SONiX AA55 / candidate",
            "Общий командный протокол AK820 Pro/MAX; GET_INFO не определяет тип переключателей и совместимость образа",
        ),
        (_, _, 0xffff, 2) if [0x3151, 0x0416].contains(&i.vid) => (
            "ROYUAN / candidate",
            "Похожие команды у нескольких поколений; одного VID/PID недостаточно",
        ),
        (0x258a, _, _, _) => (
            "BY Tech / candidate",
            "Другая ветка AK820/AULA. Нужны её пакет и протокол",
        ),
        (0x320f, _, 0xff1c, _) => (
            "EVision / candidate",
            "Другой контроллер AK820. Прошивка SONiX несовместима",
        ),
        (_, _, 0xff60, 0x61) => (
            "VIA/Vial raw HID / candidate",
            "Протокол конфигурации; наличие VIA не означает совместимость прошивки QMK",
        ),
        _ => (
            "HID / unclassified",
            "Пассивно сохранены дескрипторы; команды настройки не отправляются",
        ),
    }
}
