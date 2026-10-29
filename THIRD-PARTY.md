# Источники и лицензии

Project-owned Rust protocol implementations are written from protocol descriptions. Third-party dependency and reference licenses remain applicable; the proprietary device-control architecture does not replace those obligations.

Backup archives use the Rust `flate2` crate (MIT OR Apache-2.0) with its default Rust backend.
The archive envelope/checksums use the existing serde/serde_json and sha2 dependencies. No vendor
binary or external archiver is shipped with the backup utility.

Эталон tests/fixtures/microsoft-lamparray-descriptor.bin извлечён без изменения байтов
из LampArrayReportDescriptor.h проекта Microsoft ArduinoHidForWindows (MIT):
https://github.com/microsoft/ArduinoHidForWindows
Copyright (c) Microsoft Corporation. All rights reserved.

Полный текст лицензии Microsoft:

MIT License

Copyright (c) Microsoft Corporation.

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

Проверка SONiX использует опубликованный MIT JSON-эталон OpenKey (BakulBd):
https://github.com/BakulBd/OpenKey
Его исходный LICENSE находится рядом с эталоном в архиве исходников. Публичный отчёт
автора об аппаратном тесте служит внешним свидетельством; собственная аппаратная
валидация нашей реализации этим не заявляется.

Другие сохранённые материалы research/sources/ имеют лицензии своих проектов.
Код из проектов без найденной лицензии не копировался в реализацию и не включён
в архив исходников. В portable-архив не входят исходники сторонних проектов,
библиотеки производителя, прошивки или установщики.

Зависимости Rust перечислены с зафиксированными версиями в Cargo.lock. В частности:
eframe/egui, hidapi, serde, serde_json, anyhow, sha2, chrono, image, rfd, toml (MIT/Apache-2.0,
разбор встроенных профилей устройств), window-vibrancy (MIT/Apache-2.0, фон Mica через DWM).
Лицензии и исходники каждой зависимости доступны в соответствующих пакетах crates.io
и в локальном Cargo registry. Тексты из кешированных пакетов Cargo.lock собраны
в THIRD-PARTY-LICENSES.txt; условия версии указаны в её Cargo.toml/LICENSE.

3D-модель клавиатуры assets/models/ak820-max.glb (SHA256 d01bfeed…c52b09): реконструкция,
подготовленная владельцем репозитория по собственным ортогональным снимкам клавиатуры
(экспорт THREE.GLTFExporter r170). Сторонних моделей и текстур в репозитории нет; логотипы
производителя в модели отсутствуют.

Иконки интерфейса: контуры из Hugeicons Free 4.3.5, стиль Stroke Rounded
(npm-пакет @hugeicons/core-free-icons, https://hugeicons.com), лицензия MIT.
В src/ui/icons/glyphs.rs встроены только используемые иконки (данные контуров SVG).

MIT License

Copyright (c) 2025 Hugeicons

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

Hugeicons Free 4.3.5 Stroke Rounded: added `BatteryEmptyIcon` in `src/ui/icons/glyphs.rs`, under the same MIT license as the existing glyphs. Source: https://cdn.jsdelivr.net/npm/@hugeicons/core-free-icons@4.3.5/dist/esm/BatteryEmptyIcon.js

AJ159 preview: `assets/models/aj159.glb` is the mouse-and-dock model supplied by the repository owner
for this integration on 2026-10-28 (GLTFExporter r170). Original supplied SHA256:
`F3FAA7EAD4185AD7FE0C74680AE88E402936974CBC1B42ADF847E7E7F6BB08FB`.
No independent authorship or redistribution license was supplied; this entry records provenance only.

The integrated model swaps the mouse/dock positions and turns the dock screen toward the viewer.
Only the two assembly transforms changed; geometry, materials and textures are preserved.
Integrated SHA256: `448A17F9ADA3AA27065355239C2FB37F903BF9E0E9D24ABD7E85E1D645DD1B83`.

Battery protocol research references: `leisiji/AulaSofwareLinux` (`src/proto.rs`, MIT) and
`deepan-alve/womier-l65-linux` (`linux/l65ctl.py`, MIT). Preserved reference files and their licenses
are under `research/sources/`; SHA256 hashes and source URLs are in `research/SOURCE-MANIFEST.json`.
Only protocol facts are used by the application's own Rust implementation; these references are
neither linked into nor executed by the application.

GPU rendering dependencies: wgpu 27.0.1 and its shader compiler naga 27.0.3 (MIT OR Apache-2.0),
pollster 0.4.0 (Apache-2.0/MIT), and bytemuck 1.25.2 (Zlib OR Apache-2.0 OR MIT). Exact transitive
versions are pinned in Cargo.lock. Licenses and upstream notices are supplied by each cached crate's
Cargo.toml and license files. The project's WGSL shaders and rendering host are original code.
