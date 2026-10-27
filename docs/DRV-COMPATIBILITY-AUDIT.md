# Проверка Downloads/Drv — 2026-10-27

Проверка файлов без запуска установщиков, приложений AJAZZ, DLL и обновляторов.
USB/HID, OpenRGB и клавиатура не опрашивались. Распаковка выполнена архиваторами
7-Zip и innoextract в `research/drv-audit/`; исходная папка не изменялась.

## Механическая AK820 Max

`AJAZZ_AK820_MAX_driver_V1.0.zip`, 1959226 байт.
SHA-256: `a264ecb265b3c1eb3d60b6138e89b785f39f83c3b668f564be22107e30b0d6cf`.

Побайтно совпадает с архивом по ссылке со страницы механической модели:
https://ajazzstore.com/products/ajazz-ak820-max
https://orders.epomaker.com/software/AJAZZ_AK820_MAX_driver_V1.0.zip

Повторно извлечено и проверено содержимое:

- `mechanical/app/Cfg.ini`: `Title=AJAZZ AK820MAX`, `Appdir=BYCOMBO4`.
- `mechanical/app/Dev/kb/Wired/KB.ini`: `VID=0x258a`, `PID=0x010c`, `Psd=3,0,0,0,1,42`.
- `mechanical/app/Dev/kb/Thr/KB.ini`: те же USB VID/PID, `VID_Wireless=0x3554`,
  `PID_Wireless=0xfa09`, `Psd=3,0,0,0,1,5e`.
- `mechanical/app/OemDrv.exe`: SHA-256
  `fc7109b0467602a7624b2a83402bae12255ad5ad2e9483a2c4750c584231c06c`.

Это пакет для механической линейки AK820 Max. Связь именно RGPWB2401 с этими
идентификаторами установлена 2026-10-28 пассивным `--probe` по USB: клавиатура
владельца определилась как `258A:010C` (`docs/DEVICES.md`).
В AJAZZ Connect эта ветка только классифицируется (`src/families.rs`);
нативного BY Tech backend в `src/lighting.rs` нет.

## Магнитная AK820 Max

`AJAZZ_AK820_MAX_Single_Mode_RGB_Magnetic_Axis_Keyboard_Driver_Installer.zip`,
17983320 байт. SHA-256:
`6a20f82e7b0c850a0eff9c5de78899a875d9466cbe9882d458a63c24803a6de6`.

Внутри установщик V2.06.01. В извлечённом `magnetic/app/AK820MAX.exe`
статически найдены `Version=V_2.06.01`,
`BuildPackageDate=V_2.06.01 - 2024.11.20 14:40:00 PM`,
`/appProfile_Magnetic.xml`, `/MagneticAxisTriggerData.xml`,
`onMagneticKeyboardAllKeysCalibrationStatusBack`,
`onTheMagneticKeyboardCalibrationPoint`.

Назначение магнитной версии подтверждается содержимым, а не только именем ZIP.
Совместимость с механической клавиатурой не подтверждена; использовать этот
пакет как профиль механической платы оснований нет. Полный список USB-ID
магнитного приложения этим статическим разбором не восстановлен.

## AJ159 APEX

`AJ159 APEX-3950 ID：1755.rar`, 22255901 байт. SHA-256:
`43e64058915b6bad1ba842077bb200b6b73d305804581d12a8e6c93bcb3e53b1`.

Содержит `Base upgrade tool.zip` и `Mouse Upgrade Tool.zip`, а не драйвер AK820.
В обоих есть `ry_upgrade.exe`, конфигурации и файлы payload `x1`–`x7`.
В `Mouse Upgrade Tool/.../resources/tool_config.json` задано
`id: 1755`, `unlimited: false`, `upgrade_methods: ["usb"]`.
В конфигурации базы есть `3151:5007` и boot ID `3151:402e`.
Это обновляторы мыши и базы/приёмника. Для AK820 Max не предназначены.

## Вывод

Из трёх архивов только AK820_MAX_driver_V1.0 является подходящим источником
протокола механической AK820 Max. Его наличие в исходных материалах не означает,
что BY Tech управление уже реализовано в AJAZZ Connect. Совместимость конкретного
экземпляра RGPWB2401 и безопасность аппаратных операций не проверялись.
