use crate::{
    db::{Prompt, PromptStore},
    system::{copy_to_clipboard, HotkeyController},
    template::{extract_variables, render_template},
};
use eframe::egui::{
    self, vec2, Align, Color32, Context, CornerRadius, Event, FontData, FontDefinitions,
    FontFamily, FontId, Key, Layout, Modifiers, Pos2, RichText, ScrollArea, Stroke, TextEdit,
    ViewportCommand,
};
use std::{collections::HashMap, path::Path, sync::Arc};

const PANEL_GAP: f32 = 14.0;
const PANEL_A_WIDTH: f32 = 360.0;
const PANEL_B_WIDTH: f32 = 520.0;
const PANEL_C_WIDTH: f32 = 360.0;
const WINDOW_MARGIN: f32 = 18.0;
const PANEL_SCREEN_HEIGHT_RATIO: f32 = 0.8;

pub struct PromptBoardApp {
    store: Option<PromptStore>,
    hotkey: Option<HotkeyController>,
    prompts: Vec<Prompt>,
    query: String,
    selected: usize,
    fill: Option<FillState>,
    previous_values: HashMap<String, String>,
    visible: bool,
    skip_focus_hide_once: bool,
    status: Option<String>,
    error: Option<String>,
}

#[derive(Clone)]
struct FillState {
    prompt: Prompt,
    variables: Vec<String>,
    values: Vec<String>,
    focused_index: usize,
}

impl PromptBoardApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_pixels_per_point(1.0);

        let mut error = None;
        if let Err(err) = configure_chinese_fonts(&cc.egui_ctx) {
            error = Some(format!("字体加载失败：{err}"));
        }

        let store = match PromptStore::open_default() {
            Ok(store) => Some(store),
            Err(err) => {
                error = Some(format!("数据库错误：{err}"));
                None
            }
        };

        let hotkey = match HotkeyController::register_default() {
            Ok(hotkey) => Some(hotkey),
            Err(err) => {
                error = Some(format!("快捷键不可用：{err}"));
                None
            }
        };

        let mut app = Self {
            store,
            hotkey,
            prompts: Vec::new(),
            query: String::new(),
            selected: 0,
            fill: None,
            previous_values: HashMap::new(),
            visible: true,
            skip_focus_hide_once: false,
            status: None,
            error,
        };
        app.refresh_prompts();
        app
    }

    fn refresh_prompts(&mut self) {
        if let Some(store) = &self.store {
            match store.search(&self.query) {
                Ok(prompts) => {
                    self.prompts = prompts;
                    self.selected = self.selected.min(self.prompts.len().saturating_sub(1));
                    self.clear_fill_if_prompt_changed();
                }
                Err(err) => self.error = Some(err.to_string()),
            }
        }
    }

    fn clear_fill_if_prompt_changed(&mut self) {
        let Some(fill) = &self.fill else {
            return;
        };
        let selected_id = self.prompts.get(self.selected).map(|prompt| prompt.id);
        if selected_id != Some(fill.prompt.id) {
            self.fill = None;
        }
    }

    fn selected_prompt(&self) -> Option<&Prompt> {
        self.prompts.get(self.selected)
    }

    fn preview_prompt(&self) -> Option<&Prompt> {
        self.fill
            .as_ref()
            .map(|fill| &fill.prompt)
            .or_else(|| self.selected_prompt())
    }

    fn preview_text(&self) -> String {
        if let Some(fill) = &self.fill {
            let pairs = fill
                .variables
                .iter()
                .cloned()
                .zip(fill.values.iter().cloned())
                .collect::<Vec<_>>();
            return render_template(&fill.prompt.content, &pairs);
        }

        self.selected_prompt()
            .map(|prompt| prompt.content.clone())
            .unwrap_or_default()
    }

    fn handle_global_hotkey(&mut self, ctx: &Context) {
        if self
            .hotkey
            .as_ref()
            .is_some_and(|hotkey| hotkey.was_pressed())
        {
            self.show(ctx);
        }
    }

    fn handle_keyboard(&mut self, ctx: &Context) {
        if ctx.input(|input| input.key_pressed(Key::Escape)) {
            if self.fill.is_some() {
                self.return_to_selection(ctx);
                return;
            }
            self.hide(ctx);
            return;
        }

        if consume_copy_shortcut(ctx) {
            self.copy_preview(ctx);
            return;
        }

        if self.fill.is_some() {
            if ctx.input(|input| input.key_pressed(Key::Enter) && !input.modifiers.shift) {
                self.focus_next_field(ctx);
            }
            return;
        }

        if ctx.input(|input| input.key_pressed(Key::ArrowDown)) {
            self.selected = (self.selected + 1).min(self.prompts.len().saturating_sub(1));
            self.fill = None;
        }
        if ctx.input(|input| input.key_pressed(Key::ArrowUp)) {
            self.selected = self.selected.saturating_sub(1);
            self.fill = None;
        }
        if ctx.input(|input| input.key_pressed(Key::Enter)) {
            self.start_filling(ctx);
        }
    }

    fn start_filling(&mut self, ctx: &Context) {
        let Some(prompt) = self.selected_prompt().cloned() else {
            return;
        };

        let variables = extract_variables(&prompt.content);
        if variables.is_empty() {
            self.copy_prompt_text(prompt.id, prompt.content);
            return;
        }

        let values = variables
            .iter()
            .map(|name| self.previous_values.get(name).cloned().unwrap_or_default())
            .collect();

        self.fill = Some(FillState {
            prompt,
            variables,
            values,
            focused_index: 0,
        });
        self.status = None;
        ctx.memory_mut(|memory| memory.request_focus(egui::Id::new("var_0")));
    }

    fn focus_next_field(&mut self, ctx: &Context) {
        let Some(fill) = &mut self.fill else {
            return;
        };

        if fill.focused_index + 1 < fill.variables.len() {
            fill.focused_index += 1;
        }

        ctx.memory_mut(|memory| {
            memory.request_focus(egui::Id::new(format!("var_{}", fill.focused_index)));
        });
    }

    fn return_to_selection(&mut self, ctx: &Context) {
        self.fill = None;
        self.status = None;
        ctx.memory_mut(|memory| memory.request_focus(egui::Id::new("search")));
    }

    fn copy_preview(&mut self, ctx: &Context) {
        let text = self.preview_text();
        if text.trim().is_empty() {
            return;
        }

        if let Some(fill) = &self.fill {
            for (name, value) in fill.variables.iter().zip(fill.values.iter()) {
                self.previous_values.insert(name.clone(), value.clone());
            }
            if let Some(store) = &self.store {
                if let Err(err) = store.increment_usage(fill.prompt.id) {
                    self.error = Some(err.to_string());
                }
            }
        } else if let Some(prompt) = self.selected_prompt() {
            if let Some(store) = &self.store {
                if let Err(err) = store.increment_usage(prompt.id) {
                    self.error = Some(err.to_string());
                }
            }
        }

        ctx.copy_text(text.clone());
        match copy_to_clipboard(&text) {
            Ok(()) => self.status = Some("已复制预览内容".to_owned()),
            Err(err) => self.error = Some(format!("剪贴板错误：{err}")),
        }
        self.refresh_prompts();
    }

    fn copy_prompt_text(&mut self, prompt_id: i64, text: String) {
        match copy_to_clipboard(&text) {
            Ok(()) => {
                self.status = Some("已复制提示词".to_owned());
                if let Some(store) = &self.store {
                    if let Err(err) = store.increment_usage(prompt_id) {
                        self.error = Some(err.to_string());
                    }
                }
                self.refresh_prompts();
            }
            Err(err) => self.error = Some(format!("剪贴板错误：{err}")),
        }
    }

    fn show(&mut self, ctx: &Context) {
        self.visible = true;
        self.skip_focus_hide_once = true;
        ctx.send_viewport_cmd(ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(ViewportCommand::Focus);
        ctx.memory_mut(|memory| memory.request_focus(egui::Id::new("search")));
    }

    fn position_window_at_right(&self, ctx: &Context) {
        let Some(monitor_size) = ctx.input(|input| input.viewport().monitor_size) else {
            return;
        };

        let panel_height = desired_panel_height(monitor_size.y);
        let height = panel_height + WINDOW_MARGIN * 2.0;
        let width = self.desired_window_width();
        let x = (monitor_size.x - width - WINDOW_MARGIN).max(WINDOW_MARGIN);
        let y = ((monitor_size.y - height) / 2.0).max(WINDOW_MARGIN);
        ctx.send_viewport_cmd(ViewportCommand::InnerSize(vec2(width, height)));
        ctx.send_viewport_cmd(ViewportCommand::OuterPosition(Pos2::new(x, y)));
    }

    fn desired_window_width(&self) -> f32 {
        let mut width = PANEL_A_WIDTH + PANEL_GAP + PANEL_B_WIDTH;
        if self.fill.is_some() {
            width += PANEL_GAP + PANEL_C_WIDTH;
        }
        width + WINDOW_MARGIN * 2.0
    }

    fn hide(&mut self, ctx: &Context) {
        self.visible = false;
        self.skip_focus_hide_once = false;
        ctx.send_viewport_cmd(ViewportCommand::Visible(false));
    }

    fn hide_when_unfocused(&mut self, ctx: &Context) {
        if self.skip_focus_hide_once {
            self.skip_focus_hide_once = false;
            return;
        }

        if self.visible && !ctx.input(|input| input.focused) {
            self.hide(ctx);
        }
    }

    fn draw_panels(&mut self, ui: &mut egui::Ui) {
        let panel_height = (ui.available_height() - WINDOW_MARGIN * 2.0).max(360.0);
        let right_edge = ui.available_width() - WINDOW_MARGIN;
        let panel_y = WINDOW_MARGIN;
        let panel_a_x = right_edge - PANEL_A_WIDTH;
        let panel_b_x = panel_a_x - PANEL_GAP - PANEL_B_WIDTH;
        let panel_c_x = panel_b_x - PANEL_GAP - PANEL_C_WIDTH;

        egui::Area::new(egui::Id::new("panel_b"))
            .fixed_pos(Pos2::new(panel_b_x, panel_y))
            .order(egui::Order::Middle)
            .show(ui.ctx(), |ui| {
                self.panel_b(ui, panel_height);
            });

        if self.fill.is_some() {
            egui::Area::new(egui::Id::new("panel_c"))
                .fixed_pos(Pos2::new(panel_c_x, panel_y))
                .order(egui::Order::Middle)
                .show(ui.ctx(), |ui| {
                    self.panel_c(ui, panel_height);
                });
        }

        egui::Area::new(egui::Id::new("panel_a"))
            .fixed_pos(Pos2::new(panel_a_x, panel_y))
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                self.panel_a(ui, panel_height);
            });
    }

    fn panel_a(&mut self, ui: &mut egui::Ui, panel_height: f32) {
        mac_panel("A", ui, PANEL_A_WIDTH, panel_height, |ui| {
            ui.label(section_title("提示词"));
            ui.add_space(10.0);

            let search = TextEdit::singleline(&mut self.query)
                .id(egui::Id::new("search"))
                .hint_text("搜索标题、标签或内容")
                .desired_width(f32::INFINITY)
                .font(FontId::proportional(18.0));

            if ui.add_sized([ui.available_width(), 42.0], search).changed() {
                self.refresh_prompts();
            }

            ui.add_space(12.0);
            self.prompt_list(ui);
        });
    }

    fn prompt_list(&mut self, ui: &mut egui::Ui) {
        let mut clicked_index = None;
        let mut double_clicked_index = None;

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if self.prompts.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(64.0);
                        ui.label(body_text("没有匹配的提示词").color(Color32::from_gray(118)));
                    });
                    return;
                }

                for (index, prompt) in self.prompts.iter().enumerate() {
                    let selected = index == self.selected;
                    let response = prompt_row(ui, prompt, selected);

                    if response.clicked() {
                        clicked_index = Some(index);
                    }
                    if response.double_clicked() {
                        double_clicked_index = Some(index);
                    }

                    ui.add_space(6.0);
                }
            });

        if let Some(index) = clicked_index {
            self.selected = index;
            self.fill = None;
        }
        if let Some(index) = double_clicked_index {
            self.selected = index;
            self.start_filling(ui.ctx());
        }
    }

    fn panel_b(&mut self, ui: &mut egui::Ui, panel_height: f32) {
        let title = self
            .preview_prompt()
            .map(|prompt| prompt.title.as_str())
            .unwrap_or("预览");
        let text = self.preview_text();

        mac_panel("B", ui, PANEL_B_WIDTH, panel_height, |ui| {
            ui.label(section_title(title));
            ui.add_space(6.0);
            if let Some(prompt) = self.preview_prompt() {
                ui.label(
                    RichText::new(&prompt.tags)
                        .font(FontId::proportional(15.0))
                        .color(Color32::from_rgb(105, 112, 122)),
                );
            }
            ui.add_space(14.0);

            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(text)
                            .font(FontId::proportional(19.0))
                            .color(Color32::from_rgb(30, 33, 36)),
                    );
                });
        });
    }

    fn panel_c(&mut self, ui: &mut egui::Ui, panel_height: f32) {
        let mut request_focus = None;
        let mut return_to_selection = false;
        let Some(fill) = &mut self.fill else {
            return;
        };

        mac_panel("C", ui, PANEL_C_WIDTH, panel_height, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .add_sized(
                        [32.0, 28.0],
                        egui::Button::new(
                            RichText::new("‹")
                                .font(FontId::proportional(25.0))
                                .color(Color32::from_rgb(42, 45, 50)),
                        ),
                    )
                    .on_hover_text("返回选择")
                    .clicked()
                {
                    return_to_selection = true;
                }
                ui.label(section_title("填写变量"));
            });
            ui.add_space(4.0);
            ui.label(
                RichText::new("Esc 返回选择，Enter 切换输入框，Command+C 复制预览内容")
                    .font(FontId::proportional(14.0))
                    .color(Color32::from_rgb(105, 112, 122)),
            );
            ui.add_space(14.0);

            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for (index, variable) in fill.variables.iter().enumerate() {
                        let field_id = egui::Id::new(format!("var_{index}"));
                        ui.label(
                            RichText::new(variable)
                                .font(FontId::proportional(16.0))
                                .strong()
                                .color(Color32::from_rgb(42, 45, 50)),
                        );

                        let previous = self.previous_values.get(variable).cloned();
                        let hint = previous
                            .as_ref()
                            .filter(|value| !value.is_empty())
                            .map(|value| format!("默认：{value}"))
                            .unwrap_or_else(|| "输入自定义内容".to_owned());

                        let response = ui.add_sized(
                            [ui.available_width(), 42.0],
                            TextEdit::singleline(&mut fill.values[index])
                                .id(field_id)
                                .hint_text(hint)
                                .font(FontId::proportional(17.0)),
                        );

                        if response.gained_focus() {
                            fill.focused_index = index;
                        }
                        if fill.focused_index == index && !response.has_focus() {
                            request_focus = Some(field_id);
                        }

                        ui.add_space(14.0);
                    }
                });
        });

        if let Some(field_id) = request_focus {
            ui.ctx().memory_mut(|memory| memory.request_focus(field_id));
        }
        if return_to_selection {
            self.return_to_selection(ui.ctx());
        }
    }
}

impl eframe::App for PromptBoardApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.handle_global_hotkey(ctx);
        self.handle_keyboard(ctx);
        self.hide_when_unfocused(ctx);
        if self.visible {
            self.position_window_at_right(ctx);
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(Color32::TRANSPARENT))
            .show(ctx, |ui| {
                if let Some(error) = &self.error {
                    egui::Area::new(egui::Id::new("error_banner"))
                        .fixed_pos(Pos2::new(18.0, 14.0))
                        .show(ui.ctx(), |ui| {
                            egui::Frame::NONE
                                .fill(Color32::from_rgba_premultiplied(255, 235, 235, 235))
                                .corner_radius(CornerRadius::same(9))
                                .inner_margin(egui::Margin::symmetric(14, 10))
                                .show(ui, |ui| {
                                    ui.label(
                                        RichText::new(error)
                                            .font(FontId::proportional(14.0))
                                            .color(Color32::from_rgb(168, 35, 35)),
                                    );
                                });
                        });
                }

                self.draw_panels(ui);

                if let Some(status) = &self.status {
                    egui::Area::new(egui::Id::new("status_banner"))
                        .anchor(egui::Align2::RIGHT_BOTTOM, vec2(-22.0, -18.0))
                        .show(ui.ctx(), |ui| {
                            egui::Frame::NONE
                                .fill(Color32::from_rgba_premultiplied(246, 248, 250, 235))
                                .stroke(Stroke::new(1.0, Color32::from_rgb(205, 210, 218)))
                                .corner_radius(CornerRadius::same(9))
                                .inner_margin(egui::Margin::symmetric(14, 10))
                                .show(ui, |ui| {
                                    ui.label(
                                        RichText::new(status)
                                            .font(FontId::proportional(14.0))
                                            .color(Color32::from_rgb(55, 61, 68)),
                                    );
                                });
                        });
                }
            });

        ctx.request_repaint_after(std::time::Duration::from_millis(80));
    }
}

fn mac_panel(
    id: &str,
    ui: &mut egui::Ui,
    width: f32,
    height: f32,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    ui.push_id(id, |ui| {
        egui::Frame::NONE
            .fill(Color32::from_rgba_premultiplied(246, 248, 250, 222))
            .stroke(Stroke::new(
                1.0,
                Color32::from_rgba_premultiplied(190, 196, 205, 150),
            ))
            .corner_radius(CornerRadius::same(14))
            .inner_margin(egui::Margin::same(18))
            .show(ui, |ui| {
                let inner_width = (width - 36.0).max(120.0);
                let inner_height = (height - 36.0).max(120.0);
                ui.set_min_width(inner_width);
                ui.set_width(inner_width);
                ui.set_min_height(inner_height);
                ui.set_height(inner_height);
                add_contents(ui);
            });
    });
}

fn desired_panel_height(monitor_height: f32) -> f32 {
    (monitor_height * PANEL_SCREEN_HEIGHT_RATIO).max(360.0)
}

fn consume_copy_shortcut(ctx: &Context) -> bool {
    ctx.input_mut(|input| {
        let mut copied = false;
        input.events.retain(|event| {
            let is_copy = matches!(event, Event::Copy);
            copied |= is_copy;
            !is_copy
        });

        copied || input.consume_key(Modifiers::COMMAND, Key::C)
    })
}

fn prompt_row(ui: &mut egui::Ui, prompt: &Prompt, selected: bool) -> egui::Response {
    let fill = if selected {
        Color32::from_rgba_premultiplied(0, 122, 255, 225)
    } else {
        Color32::from_rgba_premultiplied(255, 255, 255, 95)
    };
    let stroke = if selected {
        Stroke::new(1.0, Color32::from_rgba_premultiplied(0, 92, 220, 210))
    } else {
        Stroke::new(1.0, Color32::from_rgba_premultiplied(215, 220, 228, 130))
    };
    let primary = if selected {
        Color32::WHITE
    } else {
        Color32::from_rgb(28, 31, 35)
    };
    let secondary = if selected {
        Color32::from_rgba_premultiplied(255, 255, 255, 210)
    } else {
        Color32::from_rgb(101, 108, 118)
    };

    egui::Frame::NONE
        .fill(fill)
        .stroke(stroke)
        .corner_radius(CornerRadius::same(9))
        .inner_margin(egui::Margin::symmetric(12, 11))
        .show(ui, |ui| {
            ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                ui.label(
                    RichText::new(&prompt.title)
                        .font(FontId::proportional(17.0))
                        .strong()
                        .color(primary),
                );
                ui.add_space(3.0);
                ui.label(
                    RichText::new(format!("{} · 使用 {}", prompt.tags, prompt.usage_count))
                        .font(FontId::proportional(14.0))
                        .color(secondary),
                );
            });
        })
        .response
}

fn section_title(text: &str) -> RichText {
    RichText::new(text)
        .font(FontId::proportional(22.0))
        .strong()
        .color(Color32::from_rgb(24, 27, 31))
}

fn body_text(text: &str) -> RichText {
    RichText::new(text)
        .font(FontId::proportional(17.0))
        .color(Color32::from_rgb(40, 44, 50))
}

fn configure_chinese_fonts(ctx: &Context) -> Result<(), String> {
    let candidates = [
        "/Library/Fonts/Arial Unicode.ttf",
        "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        "/System/Library/Fonts/STHeiti Medium.ttc",
    ];

    let Some((font_path, font_bytes)) = candidates
        .iter()
        .find_map(|path| read_font(path).map(|bytes| (*path, bytes)))
    else {
        return Err("没有找到可用的系统中文字体".to_owned());
    };

    let mut fonts = FontDefinitions::default();
    let font_name = "system_chinese".to_owned();
    fonts.font_data.insert(
        font_name.clone(),
        Arc::new(FontData::from_owned(font_bytes)),
    );

    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, font_name.clone());
    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .insert(0, font_name);

    ctx.set_fonts(fonts);
    eprintln!("Loaded Chinese font: {font_path}");
    Ok(())
}

fn read_font(path: &str) -> Option<Vec<u8>> {
    std::fs::read(Path::new(path)).ok()
}
