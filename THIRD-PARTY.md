# Источники и лицензии

Реализации Rust в src/ написаны в этом проекте по форматам протоколов, а не скопированы
из OpenRGB или других проектов. OpenRGB не встроен в EXE: используется опубликованный
TCP SDK. Сохранённые исходники OpenRGB не компилируются в приложение и остаются
под своей GPL.

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
eframe/egui, hidapi, serde, serde_json, anyhow, sha2, chrono, image, rfd.
Лицензии и исходники каждой зависимости доступны в соответствующих пакетах crates.io
и в локальном Cargo registry. Тексты из кешированных пакетов Cargo.lock собраны
в THIRD-PARTY-LICENSES.txt; условия версии указаны в её Cargo.toml/LICENSE.

