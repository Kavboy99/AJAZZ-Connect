#![cfg_attr(windows, windows_subsystem = "windows")]
use ak820max_assistant::firmware_assistant::{self as assistant, PackageInfo, Report};
use anyhow::{Context, Result};
use eframe::egui::{self, Color32, RichText};
use std::{path::Path, sync::mpsc};

const INK: Color32 = Color32::from_rgb(32, 38, 47);
const MUTED: Color32 = Color32::from_rgb(108, 116, 128);
const ACCENT: Color32 = Color32::from_rgb(0, 111, 214);

enum Event {
    Probe(Result<Report, String>),
    Package(Result<PackageInfo, String>),
    LightConnected(Result<Box<ak820max_assistant::lighting::Session>, String>),
    LightDone(
        Box<ak820max_assistant::lighting::Session>,
        Result<String, String>,
    ),
}
struct App {
    report: Option<Report>,
    package: Option<PackageInfo>,
    pending: bool,
    notice: String,
    tx: mpsc::Sender<Event>,
    rx: mpsc::Receiver<Event>,
    page: u8,
    backend: u8,
    light: Option<Box<ak820max_assistant::lighting::Session>>,
    selected: usize,
    rgb: [u8; 3],
    // Only --review fills this; synthetic UI states never open HID or network.
    preview: Option<Vec<ak820max_assistant::lighting::Target>>,
}
impl App {
    fn new(ctx: &egui::Context) -> Self {
        configure(ctx);
        let (tx, rx) = mpsc::channel();
        Self {
            report: None,
            package: None,
            pending: false,
            notice: String::new(),
            tx,
            rx,
            page: 0,
            backend: 2,
            light: None,
            selected: 0,
            rgb: [48, 140, 255],
            preview: None,
        }
    }
    fn probe(&mut self) {
        if self.pending {
            return;
        }
        self.pending = true;
        self.notice.clear();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let value = assistant::probe().map_err(|e| format!("{e:#}"));
            let _ = tx.send(Event::Probe(value));
        });
    }
    fn save(&mut self) {
        let Some(mut report) = self.report.clone() else {
            return;
        };
        report.firmware = self.package.clone();
        if let Some(path) = rfd::FileDialog::new()
            .set_file_name("AK820MAX-GPWB2401.json")
            .add_filter("Отчёт", &["json"])
            .save_file()
        {
            self.notice = match report.save(&path) {
                Ok(()) => {
                    "Отчёт сохранён. Пришлите этот JSON в чат для подготовки профиля GPWB2401."
                        .into()
                }
                Err(e) => format!("Не удалось сохранить отчёт: {e:#}"),
            };
        }
    }
    fn load_package(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Пакет прошивки", &["exe", "bin"])
            .pick_file()
        else {
            return;
        };
        self.pending = true;
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(Event::Package(
                assistant::inspect_file(&path).map_err(|e| format!("{e:#}")),
            ));
        });
    }
    fn light_connect(&mut self) {
        self.pending = true;
        self.light = None;
        self.notice.clear();
        self.selected = 0;
        let tx = self.tx.clone();
        let backend = self.backend;
        std::thread::spawn(move || {
            let result = ak820max_assistant::lighting::Session::connect(backend)
                .map(Box::new)
                .map_err(|e| format!("{e:#}"));
            let _ = tx.send(Event::LightConnected(result));
        });
    }
    fn light_apply(&mut self, restore: bool, file: Option<std::path::PathBuf>) {
        let Some(mut session) = self.light.take() else {
            return;
        };
        self.pending = true;
        let tx = self.tx.clone();
        let selected = self.selected;
        let rgb = self.rgb;
        std::thread::spawn(move || {
            let result = if let Some(path) = file {
                session.restore_file(&path)
            } else if restore {
                session.restore(selected)
            } else {
                session.solid(selected, rgb)
            };
            let _ = tx.send(Event::LightDone(
                session,
                result.map_err(|e| format!("{e:#}")),
            ));
        });
    }
    fn lighting_ui(&mut self, ui: &mut egui::Ui) {
        card(ui, |ui| {
            ui.label(
                RichText::new("Подсветка разных клавиатур")
                    .size(20.0)
                    .strong(),
            );
            ui.label("Управление цветом без замены прошивки. Сначала выберите способ подключения.");
            ui.add_space(8.0);
            ui.add_enabled_ui(!self.pending, |ui| {
                let previous=self.backend;
                ui.horizontal_wrapped(|ui| {
                    ui.radio_value(&mut self.backend, 2, "AJAZZ · AA55");
                    ui.radio_value(&mut self.backend, 3, "SONiX Feature");
                    ui.radio_value(&mut self.backend, 0, "HID LampArray");
                    ui.radio_value(&mut self.backend, 1, "OpenRGB SDK");
                });
                if previous!=self.backend {self.light=None;self.selected=0;self.notice.clear();}
                if self.backend==1 {
                    ui.label(RichText::new("Откройте OpenRGB → SDK Server → Start Server (порт 6742). Клавиатура должна определяться в самом OpenRGB.").color(MUTED));
                } else if self.backend==0 {
                    ui.label(RichText::new("Работает напрямую с клавиатурами стандарта HID LampArray / Dynamic Lighting. Фирменный драйвер не требуется.").color(MUTED));
                } else if self.backend==2 {
                    ui.label(RichText::new("Штатный протокол AK820 Pro/MAX. Подключение проверяется по дескрипторам, GET_INFO и LED_EFFECT. Версия прошивки не фиксируется в коде.").color(MUTED));
                } else {
                    ui.label(RichText::new("Экспериментальный адаптер старых AK820/MAX/PLUS с Feature-интерфейсом. Настройка сохраняется в клавиатуре; исходный эффект прочитать и автоматически вернуть нельзя.").color(MUTED));
                }
                if ui.button("Найти клавиатуры").clicked() { self.light_connect(); }
            });
        });
        ui.add_space(12.0);
        let targets = self
            .light
            .as_ref()
            .map(|s| s.targets())
            .or_else(|| self.preview.clone())
            .unwrap_or_default();
        let notes = self.light.as_ref().map(|s| s.notes()).unwrap_or_default();
        let restorable = self.light.as_ref().is_some_and(|s| s.restorable());
        let file_restore = self.light.as_ref().is_some_and(|s| s.file_restore());
        card(ui, |ui| {
            if targets.is_empty() {
                ui.label(
                    RichText::new(if self.light.is_some() {
                        "Подходящие клавиатуры не найдены."
                    } else {
                        "После подключения здесь появятся доступные клавиатуры."
                    })
                    .color(MUTED),
                );
                if self.light.is_some() {
                    ui.label("Для AJAZZ с собственным протоколом откройте «Диагностика»: приложение определит семейство интерфейсов.");
                }
            } else {
                let selected = self.selected.min(targets.len() - 1);
                self.selected = selected;
                egui::ComboBox::from_id_salt("keyboard")
                    .selected_text(&targets[selected].label)
                    .show_ui(ui, |ui| {
                        for (index, target) in targets.iter().enumerate() {
                            ui.selectable_value(&mut self.selected, index, &target.label);
                        }
                    });
                ui.label(RichText::new(&targets[self.selected].detail).color(MUTED));
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    ui.label("Цвет");
                    ui.color_edit_button_srgb(&mut self.rgb);
                    ui.label(format!(
                        "#{:02X}{:02X}{:02X}",
                        self.rgb[0], self.rgb[1], self.rgb[2]
                    ));
                });
                ui.add_space(10.0);
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .add_enabled(
                            !self.pending && targets[self.selected].writable,
                            egui::Button::new("Применить цвет"),
                        )
                        .clicked()
                    {
                        self.light_apply(false, None);
                    }
                    if ui
                        .add_enabled(
                            !self.pending && restorable,
                            egui::Button::new("Вернуть подсветку"),
                        )
                        .clicked()
                    {
                        self.light_apply(true, None);
                    }
                    if file_restore
                        && ui
                            .add_enabled(!self.pending, egui::Button::new("Из резервной копии…"))
                            .clicked()
                        && let Some(path) = rfd::FileDialog::new()
                            .add_filter("Резервная копия подсветки", &["json"])
                            .pick_file()
                    {
                        self.light_apply(true, Some(path));
                    }
                });
                ui.label(RichText::new("AA55 и OpenRGB сохраняют исходное состояние перед первым изменением. LampArray возвращается к автономной подсветке кнопкой или при закрытии приложения.").size(12.0).color(MUTED));
            }
            if !notes.is_empty() {
                ui.add_space(8.0);
                ui.label(RichText::new(&notes).size(12.0).color(MUTED));
            }
        });
        ui.add_space(12.0);
        card(ui, |ui| {
            ui.label(RichText::new("AK820MAX · GPWB2401").strong());
            ui.label("Название модели не определяет контроллер. SONiX Feature, AA55, ROYUAN и BY Tech требуют разных адаптеров. Диагностика покажет найденное семейство; совместимость GPWB2401 пока не подтверждена.");
        });
    }
    fn ui(&mut self, ctx: &egui::Context) {
        while let Ok(event) = self.rx.try_recv() {
            self.pending = false;
            match event {
                Event::Probe(Ok(report)) => self.report = Some(report),
                Event::Package(Ok(package)) => {
                    self.package = Some(package);
                    self.notice.clear();
                }
                Event::Probe(Err(e)) | Event::Package(Err(e)) => self.notice = e,
                Event::LightConnected(result) => match result {
                    Ok(session) => {
                        self.light = Some(session);
                        self.notice.clear();
                    }
                    Err(e) => self.notice = e,
                },
                Event::LightDone(session, result) => match result {
                    Ok(message) => {
                        self.light = Some(session);
                        self.notice = message;
                    }
                    Err(e) => {
                        let hint = if session.file_restore() {
                            "Созданные до записи резервные копии доступны в %LOCALAPPDATA%\\AjazzConnect\\backups."
                        } else {
                            ""
                        };
                        self.notice = format!("{e}\nПодключитесь заново. {hint}");
                    }
                },
            }
        }
        egui::CentralPanel::default().frame(egui::Frame::new()
            .fill(Color32::from_rgb(246,248,251)).inner_margin(28.0)).show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.label(RichText::new("AJAZZ").size(12.0).strong().color(MUTED));
                ui.add_space(5.0);
                ui.label(RichText::new("AJAZZ Connect").size(32.0).strong().color(INK));
                ui.label(RichText::new("Подсветка · семейства контроллеров · анализ прошивок").size(15.0).color(MUTED));
                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.page,0,"Подсветка");
                    ui.selectable_value(&mut self.page,1,"Диагностика и прошивка");
                });
                ui.add_space(12.0);
                if self.page == 0 {
                    self.lighting_ui(ui);
                    if self.pending {ui.spinner();ui.label("Обмен с устройством…");}
                    if !self.notice.is_empty() {ui.add_space(12.0);ui.label(RichText::new(&self.notice).color(ACCENT));}
                    return;
                }
                card(ui, |ui| {
                    ui.label(RichText::new("1. Подключите клавиатуру").size(18.0).strong());
                    ui.add_space(8.0);
                    ui.label("Подключите клавиатуру кабелем и выберите проводной режим. Закройте другие драйверы клавиатуры.");
                    ui.add_space(13.0);
                    ui.horizontal_wrapped(|ui| {
                        if ui.add_enabled(!self.pending, egui::Button::new("Проверить подключение")
                            .min_size(egui::vec2(190.0,35.0))).clicked() { self.probe(); }
                        if self.pending { ui.spinner(); ui.label(RichText::new("Чтение данных…").color(MUTED)); }
                    });
                    if let Some(report) = &self.report {
                        ui.add_space(14.0);
                        if let Some(info) = report.wired_info() {
                            ui.label(RichText::new(format!("USB {:04X}:{:04X} · прошивка {}",info.vid,info.pid,info.version)).strong().color(ACCENT));
                            ui.label(RichText::new(format!("Manufacturer {:04X} · Product {:04X}",info.manufacturer,info.product)).size(13.0).color(MUTED));
                        } else if report.interfaces.iter().any(|i| i.vid == 0x0c45 && i.pid == 0xfefc) {
                            ui.label(RichText::new("Виден радиоприёмник. Нужен проводной режим USB.").color(MUTED));
                        } else if report.interfaces.is_empty() {
                            ui.label(RichText::new("Клавиатура пока не найдена. Проверьте кабель и режим подключения.").color(MUTED));
                        } else {
                            ui.label("Найдены HID-интерфейсы. Их данные будут включены в отчёт.");
                        }
                        let errors: Vec<_> = report.interfaces.iter().filter_map(|i| i.read_error.as_deref()).collect();
                        if !errors.is_empty() {
                            ui.add_space(7.0);
                            ui.label(RichText::new(errors[0]).size(13.0).color(MUTED));
                        }
                        egui::CollapsingHeader::new("Интерфейсы и семейства протоколов").show(ui, |ui| {
                            for interface in &report.interfaces {
                                ui.label(RichText::new(format!("{:04X}:{:04X} · {:04X}:{:04X} · {}",interface.vid,interface.pid,interface.usage_page,interface.usage,interface.protocol_family)).strong());
                                ui.label(RichText::new(&interface.control_hint).size(12.0).color(MUTED));
                            }
                        });
                    }
                });
                ui.add_space(12.0);
                card(ui, |ui| {
                    ui.label(RichText::new("2. Сохраните данные ревизии").size(18.0).strong());
                    ui.add_space(8.0);
                    ui.label("Маркировка GPWB2401 ещё не сопоставлена с прошивкой. Отчёт позволит выбрать образ для этой платы.");
                    ui.add_space(12.0);
                    if ui.add_enabled(self.report.is_some() && !self.pending,
                        egui::Button::new("Сохранить отчёт…").min_size(egui::vec2(170.0,35.0))).clicked() { self.save(); }
                });
                ui.add_space(12.0);
                card(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Прошивка").size(18.0).strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.add_enabled(!self.pending, egui::Button::new("Выбрать пакет…")).clicked() { self.load_package(); }
                        });
                    });
                    ui.add_space(8.0);
                    if let Some(package) = &self.package {
                        let id = match (package.image.vid,package.image.pid) {
                            (Some(vid),Some(pid)) => format!("{:04X}:{:04X}",vid,pid),
                            _ => "ID не установлен".into(),
                        };
                        ui.label(format!("{} · {} байт",id,package.image.bytes));
                        ui.label(RichText::new(format!("SHA256 {}…",&package.image.sha256[..16])).size(12.0).color(MUTED));
                        if package.advertised_checksum_matches == Some(false) {
                            ui.colored_label(Color32::from_rgb(165,57,47),"Контрольная сумма пакета не совпала.");
                        }
                    } else { ui.label(RichText::new("Можно проверить EXE или BIN от производителя. Файл не запускается.").color(MUTED)); }
                    ui.add_space(12.0);
                    ui.add_enabled(false,egui::Button::new("Обновить клавиатуру").min_size(egui::vec2(190.0,35.0)));
                    ui.add_space(7.0);
                    ui.label(RichText::new("Для прошивки GPWB2401 нужны подтверждённый образ, контроллер и карта памяти. Доступные операции этой вкладки: диагностика и статическая проверка файла.").size(13.0).color(MUTED));
                });
                if !self.notice.is_empty() {
                    ui.add_space(14.0); ui.label(RichText::new(&self.notice).color(ACCENT));
                }
                ui.add_space(18.0);
                ui.label(RichText::new("В отчёт не входят серийный номер, нажатия клавиш и личные файлы.").size(12.0).color(MUTED));
            });
        });
        if self.pending {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.ui(ctx);
    }
}
fn card(ui: &mut egui::Ui, draw: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(Color32::WHITE)
        .corner_radius(12)
        .inner_margin(18.0)
        .stroke(egui::Stroke::new(1.0_f32, Color32::from_rgb(230, 234, 239)))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            draw(ui);
        });
}
fn configure(ctx: &egui::Context) {
    ctx.set_visuals(egui::Visuals::light());
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(9.0, 8.0);
    style
        .text_styles
        .insert(egui::TextStyle::Body, egui::FontId::proportional(14.0));
    style
        .text_styles
        .insert(egui::TextStyle::Button, egui::FontId::proportional(14.0));
    ctx.set_style(style);
    if let Some(windir) = std::env::var_os("WINDIR")
        && let Ok(data) = std::fs::read(std::path::PathBuf::from(windir).join("Fonts/segoeui.ttf"))
    {
        let mut fonts = egui::FontDefinitions::default();
        fonts
            .font_data
            .insert("system-ui".into(), egui::FontData::from_owned(data).into());
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "system-ui".into());
        ctx.set_fonts(fonts);
    }
}

fn render_review(directory: &Path) -> Result<()> {
    use std::collections::BTreeMap;
    std::fs::create_dir_all(directory)?;
    for (name, with_report) in [
        ("connect", false),
        ("radio", true),
        ("aa55", false),
        ("feature", false),
    ] {
        let ctx = egui::Context::default();
        let mut app = App::new(&ctx);
        if name == "aa55" || name == "feature" {
            app.backend = if name == "aa55" { 2 } else { 3 };
            app.preview = Some(vec![ak820max_assistant::lighting::Target {
                label: "AK820 MAX · макет интерфейса".into(),
                detail: if name == "aa55" {
                    "Штатный AA55 · пример состояния".into()
                } else {
                    "SONiX Feature · экспериментально · пример состояния".into()
                },
                writable: true,
            }]);
            app.notice = "Макет для проверки разметки. Обращений к клавиатуре нет.".into();
        }
        if with_report {
            app.page = 1;
            let mut report = Report::empty();
            report.interfaces.push(assistant::Interface {
                id: "review".into(),
                vid: 0x0c45,
                pid: 0xfefc,
                product: "AJAZZ AK820MAX".into(),
                usage_page: 0xff80,
                usage: 1,
                interface: 2,
                input_bytes: Some(33),
                output_bytes: Some(33),
                feature_bytes: None,
                info: None,
                info_payload_hex: None,
                read_error: None,
                protocol_family: "AJAZZ RF receiver".into(),
                control_hint: "Радиоприёмник: его прошивка отличается от прошивки клавиатуры"
                    .into(),
                bcd_device: 0,
                report_descriptor_hex: None,
                report_descriptor_sha256: None,
            });
            app.report = Some(report);
        }
        let mut textures: BTreeMap<u64, image::RgbaImage> = BTreeMap::new();
        let mut filters = BTreeMap::new();
        let mut final_output = None;
        for _ in 0..4 {
            let out = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(700.0, 780.0),
                    )),
                    ..Default::default()
                },
                |ctx| app.ui(ctx),
            );
            for (id, delta) in &out.textures_delta.set {
                let egui::TextureId::Managed(id) = id else {
                    anyhow::bail!("Native texture");
                };
                let egui::ImageData::Color(data) = &delta.image;
                let raw: Vec<u8> = data.pixels.iter().flat_map(|p| p.to_array()).collect();
                let patch =
                    image::RgbaImage::from_raw(data.size[0] as u32, data.size[1] as u32, raw)
                        .context("Image size")?;
                if let Some([x, y]) = delta.pos {
                    image::imageops::replace(
                        textures.get_mut(id).context("Atlas")?,
                        &patch,
                        x as i64,
                        y as i64,
                    );
                } else {
                    textures.insert(*id, patch);
                }
                filters.insert(
                    *id,
                    delta.options.magnification == egui::TextureFilter::Nearest,
                );
            }
            final_output = Some(out);
        }
        let out = final_output.context("No frame")?;
        let mut meshes = Vec::new();
        for p in ctx.tessellate(out.shapes, out.pixels_per_point) {
            let egui::epaint::Primitive::Mesh(mesh) = p.primitive else {
                anyhow::bail!("Native callback");
            };
            let egui::TextureId::Managed(id) = mesh.texture_id else {
                anyhow::bail!("Native texture");
            };
            let vertices: Vec<_> = mesh
                .vertices
                .iter()
                .map(|v| serde_json::json!([v.pos.x, v.pos.y, v.uv.x, v.uv.y, v.color.to_array()]))
                .collect();
            meshes.push(serde_json::json!({"texture":id,"clip":[p.clip_rect.min.x,p.clip_rect.min.y,p.clip_rect.max.x,p.clip_rect.max.y],"vertices":vertices,"indices":mesh.indices}));
        }
        for (id, texture) in textures {
            texture.save(directory.join(format!("{name}-texture-{id}.png")))?;
        }
        std::fs::write(
            directory.join(format!("{name}.json")),
            serde_json::to_vec(
                &serde_json::json!({"width":700,"height":780,"scale":out.pixels_per_point,"nearest":filters,"meshes":meshes,"hardware_access":false,"native_windows":false}),
            )?,
        )?;
    }
    Ok(())
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().collect();
    if let Some(command) = args.get(1).and_then(|x| x.to_str()) {
        match command {
            "--probe" => {
                let path = Path::new(args.get(2).context("Укажите путь отчёта JSON")?);
                return assistant::probe()?.save(path);
            }
            "--inspect" => {
                let input = Path::new(args.get(2).context("Укажите EXE или BIN")?);
                let output = Path::new(args.get(3).context("Укажите путь отчёта JSON")?);
                return ak820max_assistant::storage::write_json_atomic(
                    output,
                    &assistant::inspect_file(input)?,
                );
            }
            "--review" => {
                return render_review(Path::new(args.get(2).context("Укажите каталог review")?));
            }
            "--lighting-probe" => {
                let name = args
                    .get(2)
                    .and_then(|s| s.to_str())
                    .context("Укажите aa55, feature, lamparray или openrgb")?;
                let kind = match name {
                    "lamparray" => 0,
                    "openrgb" => 1,
                    "aa55" => 2,
                    "feature" => 3,
                    _ => anyhow::bail!("Неизвестный адаптер"),
                };
                let output = Path::new(args.get(3).context("Укажите JSON")?);
                let result = ak820max_assistant::lighting::Session::connect(kind);
                let value = match result {
                    Ok(session) => {
                        serde_json::json!({"backend":name,"version":ak820max_assistant::version(),"devices":session.targets().iter().map(|t|serde_json::json!({"label":t.label,"detail":t.detail,"writable":t.writable})).collect::<Vec<_>>(),"notes":session.notes(),"firmware_writes":false,"settings_writes":false,"session_handshake":kind==3})
                    }
                    Err(e) => {
                        serde_json::json!({"backend":name,"error":format!("{e:#}"),"firmware_writes":false,"settings_writes":false})
                    }
                };
                return ak820max_assistant::storage::write_json_atomic(output, &value);
            }
            _ => anyhow::bail!(
                "Доступны --probe JSON, --inspect FILE JSON, --review DIRECTORY, --lighting-probe BACKEND JSON"
            ),
        }
    }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([700.0, 780.0])
            .with_min_inner_size([580.0, 560.0]),
        ..Default::default()
    };
    eframe::run_native(
        "AJAZZ Connect",
        options,
        Box::new(|cc| {
            let mut app = App::new(&cc.egui_ctx);
            app.probe();
            Ok(Box::new(app))
        }),
    )
    .map_err(|e| anyhow::anyhow!(e.to_string()))
}
