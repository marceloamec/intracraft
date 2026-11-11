//! The code of a code block (S21 §4): its editor in the Design module, and what the code printed and holds in the
//! Discrete simulation module.
//!
//! The editor is a window over the views (A17 §19):
//!
//! - the code coloured as Python is (keywords, events, units, the layout's objects), with line numbers and a zoom;
//! - a check against the layout while typing, the problem lines underlined and listed; a click on a problem goes to
//!   its line;
//! - completion: after `Roller3.` its members, after `@` the events, Ctrl+Space anywhere (`intracraft_script::complete`);
//! - Enter keeps the indentation (one level more after `:`), Tab and Shift+Tab indent and outdent the selected lines,
//!   Ctrl+/ comments them out and in;
//! - find and replace (Ctrl+F, Ctrl+H), go to a line (Ctrl+G), the Insert menu of event and code templates;
//! - the names the code can use, inserted at the cursor with a click; the line and column of the cursor.
//!
//! Saving runs BIMCraft's `modify` command through `intracraft_model::set_attribute`, so it can be undone; code with
//! errors can be saved, it only keeps a run from starting. The text operations are in `code/edit.rs`.

mod edit;

use std::collections::BTreeSet;

use bimcraft_core::model::Id;
use bimcraft_ui_egui::{BimApp, Status};
use egui::text::{CCursor, LayoutJob};
use egui::text_edit::TextEditState;
use egui::text_selection::CCursorRange;
use egui::{Color32, FontId, Key, Modifiers, RichText, ScrollArea, Sense, Stroke, TextFormat, Ui};
use intracraft_des::flow::{FlowSimulation, describe};
use intracraft_library::Library;
use intracraft_library::definition::Value;
use intracraft_library::layout::Layout;
use intracraft_script::compile::DECORATORS;
use intracraft_script::complete::{Block, Completion, complete};
use intracraft_script::{Diagnostic, MemberKind, ModelInfo, Source, check};

use edit::{SNIPPETS, Selection};

/// Python's keywords, coloured as such.
const KEYWORDS: [&str; 22] = [
    "def", "return", "if", "elif", "else", "for", "while", "in", "not", "and", "or", "is", "global", "pass", "break", "continue", "import",
    "from", "assert", "True", "False", "None",
];
/// Lines of the console the run panel shows, the latest.
const CONSOLE_LINES: usize = 200;
/// The code's letter size by default, and the smallest and largest the zoom gives.
const FONT_SIZE: f32 = 13.0;
const FONT_MIN: f32 = 9.0;
const FONT_MAX: f32 = 28.0;
/// The text box's id, so its cursor can be read and set.
const EDIT_ID: &str = "intrascript-code-edit";

/// The editor of one code block at a time.
#[derive(Default)]
pub struct CodeEditor {
    /// The block being edited (its element), while the window is open.
    pub element: Option<Id>,
    pub text: String,
    saved: String,
    diagnostics: Vec<Diagnostic>,
    checked: Option<String>,
    model: ModelInfo,
    /// The other code blocks (object, code), for completing their names.
    others: Vec<(u32, String)>,
    /// The selection the text box last had, as char indices (start, end).
    cursor: Selection,
    /// A selection to give the text box (after an edit, a jump or a find).
    pending: Option<Selection>,
    /// The text box had the keyboard at the last frame: the editor's keys apply.
    focused: bool,
    /// The completion list while it is open, and the item picked in it.
    completion: Option<(Completion, usize)>,
    find: Option<FindBar>,
    /// The line typed in the Go to line bar, while it is open.
    goto: Option<String>,
    /// The code's letter size; 0 means the default.
    font_size: f32,
}

/// The Find (and Replace) bar.
#[derive(Default)]
struct FindBar {
    query: String,
    with: String,
    case: bool,
    replace: bool,
    /// The query field takes the keyboard at the next frame.
    focus: bool,
}

/// What a click in the Find bar asked for.
enum FindAction {
    Next(bool),
    Replace,
    ReplaceAll,
    Close,
}

impl CodeEditor {
    /// Opens the editor on a block and its code.
    pub fn open(&mut self, element: Id, code: &str) {
        self.element = Some(element);
        self.text = code.to_string();
        self.saved = code.to_string();
        self.checked = None;
        self.cursor = (0, 0);
        self.pending = None;
        self.completion = None;
    }

    /// The problems of the code as typed now.
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Checks the typed code against the layout the model holds now (only when the code changed).
    pub fn check(&mut self, app: &BimApp, library: &Library) {
        let Some(element) = self.element else { return };
        if self.checked.as_deref() == Some(self.text.as_str()) {
            return;
        }
        let (layout, _) = intracraft_model::layout::layout(&app.session.model, library);
        let blocks = describe(&layout, library);
        let id = intracraft_model::layout::instance_id(element);
        let object = layout.instances.iter().position(|i| i.id == id).map_or(u32::MAX, |i| i as u32);
        let others: Vec<Source> =
            blocks.sources.iter().filter(|(o, _)| *o != object).map(|(o, code)| Source { object: *o, code: code.as_str() }).collect();
        self.diagnostics = check(&blocks.model, object, &self.text, &others);
        self.others = blocks.sources.iter().filter(|(o, _)| *o != object).map(|(o, code)| (*o, code.clone())).collect();
        self.model = blocks.model;
        self.checked = Some(self.text.clone());
    }

    /// Saves the code into the block: one undo step.
    pub fn save(&mut self, app: &mut BimApp, library: &Library) {
        let Some(element) = self.element else { return };
        let r = intracraft_model::set_attribute(&mut app.session, library, element, "source", &Value::Text(self.text.clone()));
        app.status = match r {
            Ok(()) => {
                self.saved = self.text.clone();
                let errors = self.diagnostics.iter().filter(|d| d.is_error()).count();
                let text = if errors == 0 {
                    "Code saved.".to_string()
                } else {
                    format!("Code saved with {errors} errors: a run won't start until they are fixed.")
                };
                Status { text, error: errors > 0 }
            }
            Err(text) => Status { text, error: true },
        };
    }

    /// The editor's window, while a block is open.
    pub fn show(&mut self, ctx: &egui::Context, app: &mut BimApp, library: &Library) {
        let Some(element) = self.element else { return };
        if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::S)) {
            self.save(app, library);
        }
        if self.focused {
            self.window_keys(ctx);
        }
        let mut open = true;
        let title = format!("Code block {element}");
        egui::Window::new(title)
            .id(egui::Id::new("intrascript-editor"))
            .open(&mut open)
            .default_size([1040.0, 680.0])
            .resizable(true)
            .show(ctx, |ui| {
                self.toolbar(ui, app, library);
                self.find_bar(ui, app);
                self.goto_bar(ui);
                ui.separator();
                ui.horizontal_top(|ui| {
                    let width = (ui.available_width() - 270.0).max(320.0);
                    ui.vertical(|ui| {
                        ui.set_width(width);
                        self.text_area(ui);
                        self.status_line(ui);
                        ui.separator();
                        self.problems(ui);
                    });
                    ui.separator();
                    ui.vertical(|ui| self.names(ui, app));
                });
            });
        if !open {
            self.element = None;
            self.completion = None;
        }
        // Pasted tabs become four spaces: the language counts spaces.
        if self.text.contains('\t') {
            self.text = self.text.replace('\t', edit::INDENT);
        }
        self.check(app, library);
    }

    fn font(&self) -> f32 {
        if self.font_size > 0.0 { self.font_size } else { FONT_SIZE }
    }

    /// The editor's shortcuts that open bars or zoom (while the text box has the keyboard).
    fn window_keys(&mut self, ctx: &egui::Context) {
        let pressed = |m: Modifiers, k: Key| ctx.input_mut(|i| i.consume_key(m, k));
        if pressed(Modifiers::COMMAND, Key::F) {
            self.open_find(false);
        }
        if pressed(Modifiers::COMMAND, Key::H) {
            self.open_find(true);
        }
        if pressed(Modifiers::COMMAND, Key::G) {
            self.goto = Some(String::new());
        }
        if pressed(Modifiers::COMMAND, Key::Plus) || pressed(Modifiers::COMMAND, Key::Equals) {
            self.zoom(1.0);
        }
        if pressed(Modifiers::COMMAND, Key::Minus) {
            self.zoom(-1.0);
        }
        if pressed(Modifiers::COMMAND, Key::Num0) {
            self.font_size = FONT_SIZE;
        }
    }

    fn zoom(&mut self, by: f32) {
        self.font_size = (self.font() + by).clamp(FONT_MIN, FONT_MAX);
    }

    fn open_find(&mut self, replace: bool) {
        let selected = self.selected_text();
        let bar = self.find.get_or_insert_with(FindBar::default);
        if !selected.is_empty() && !selected.contains('\n') {
            bar.query = selected;
        }
        bar.replace |= replace;
        bar.focus = true;
    }

    fn selected_text(&self) -> String {
        let (a, b) = self.cursor;
        self.text.chars().skip(a).take(b.saturating_sub(a)).collect()
    }

    /// Puts an edit's result in place: the new code and the selection the text box gets next.
    fn apply(&mut self, (text, sel): (String, Selection)) {
        self.text = text;
        self.pending = Some(sel);
    }

    fn toolbar(&mut self, ui: &mut Ui, app: &mut BimApp, library: &Library) {
        ui.horizontal(|ui| {
            let dirty = self.text != self.saved;
            if ui.button(if dirty { "Save (Ctrl+S) •" } else { "Save (Ctrl+S)" }).clicked() {
                self.save(app, library);
            }
            if ui.add_enabled(dirty, egui::Button::new("Revert")).on_hover_text("Back to the saved code").clicked() {
                self.text = self.saved.clone();
            }
            ui.separator();
            let mut chosen = None;
            ui.menu_button("Insert…", |ui| {
                for (k, s) in SNIPPETS.iter().enumerate() {
                    if ui.button(s.label).on_hover_text(s.help).clicked() {
                        chosen = Some(k);
                        ui.close();
                    }
                }
            })
            .response
            .on_hover_text("Insert an event handler or a piece of code at the cursor");
            if let Some(s) = chosen.and_then(|k| SNIPPETS.get(k)) {
                let result = edit::insert_snippet(&self.text, self.cursor, s);
                self.apply(result);
            }
            if ui.button("Find").on_hover_text("Find and replace (Ctrl+F, Ctrl+H)").clicked() {
                self.open_find(false);
            }
            if ui.button("Go to line").on_hover_text("Ctrl+G").clicked() {
                self.goto = Some(String::new());
            }
            if ui.button("# Comment").on_hover_text("Comment the selected lines out or in (Ctrl+/)").clicked() {
                let result = edit::toggle_comment(&self.text, self.cursor);
                self.apply(result);
            }
            if ui.small_button("A−").on_hover_text("Smaller letters (Ctrl+-)").clicked() {
                self.zoom(-1.0);
            }
            if ui.small_button("A+").on_hover_text("Bigger letters (Ctrl++)").clicked() {
                self.zoom(1.0);
            }
            ui.separator();
            let errors = self.diagnostics.iter().filter(|d| d.is_error()).count();
            let warnings = self.diagnostics.len() - errors;
            let (text, colour) = match (errors, warnings) {
                (0, 0) => ("No problems".to_string(), Color32::from_rgb(60, 150, 80)),
                (0, w) => (format!("{w} warnings"), Color32::from_rgb(200, 140, 40)),
                (e, w) => (format!("{e} errors, {w} warnings: a run won't start"), Color32::from_rgb(200, 60, 60)),
            };
            ui.label(RichText::new(text).color(colour));
        });
    }

    fn find_bar(&mut self, ui: &mut Ui, app: &mut BimApp) {
        let Some(bar) = &mut self.find else { return };
        let matches = edit::find_all(&self.text, &bar.query, bar.case);
        let mut action = None;
        ui.horizontal(|ui| {
            ui.label("Find");
            let field = ui.add(egui::TextEdit::singleline(&mut bar.query).desired_width(180.0).id(egui::Id::new("intrascript-find")));
            if bar.focus {
                field.request_focus();
                bar.focus = false;
            }
            if field.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                action = Some(FindAction::Next(false));
            }
            ui.checkbox(&mut bar.case, "Aa").on_hover_text("Match upper and lower case");
            ui.label(format!("{} found", matches.len()));
            if ui.small_button("Previous").clicked() {
                action = Some(FindAction::Next(true));
            }
            if ui.small_button("Next").on_hover_text("Enter").clicked() {
                action = Some(FindAction::Next(false));
            }
            if bar.replace {
                ui.label("Replace with");
                ui.add(egui::TextEdit::singleline(&mut bar.with).desired_width(140.0));
                if ui.button("Replace").clicked() {
                    action = Some(FindAction::Replace);
                }
                if ui.button("Replace all").clicked() {
                    action = Some(FindAction::ReplaceAll);
                }
            } else if ui.small_button("Replace…").clicked() {
                bar.replace = true;
            }
            if ui.small_button("Close").on_hover_text("Esc").clicked() || ui.input(|i| i.key_pressed(Key::Escape)) {
                action = Some(FindAction::Close);
            }
        });
        let (query, with, case) = (bar.query.clone(), bar.with.clone(), bar.case);
        match action {
            Some(FindAction::Next(back)) => {
                let from = if back { self.cursor.0 } else { self.cursor.1 };
                self.pending = edit::next_match(&matches, from, back);
            }
            Some(FindAction::Replace) => {
                let here = matches.iter().find(|m| **m == self.cursor).copied();
                if let Some(sel) = here {
                    let (text, after) = edit::replace(&self.text, sel, &with);
                    let next = edit::next_match(&edit::find_all(&text, &query, case), after.1, false);
                    self.text = text;
                    self.pending = next.or(Some(after));
                } else {
                    self.pending = edit::next_match(&matches, self.cursor.0, false);
                }
            }
            Some(FindAction::ReplaceAll) => {
                let (text, n) = edit::replace_all(&self.text, &query, &with, case);
                self.text = text;
                self.pending = Some((0, 0));
                app.status = Status { text: format!("Replaced {n} times."), error: false };
            }
            Some(FindAction::Close) => self.find = None,
            None => {}
        }
    }

    fn goto_bar(&mut self, ui: &mut Ui) {
        let Some(line) = &mut self.goto else { return };
        let mut go = None;
        let mut close = false;
        ui.horizontal(|ui| {
            ui.label("Go to line");
            let field = ui.add(egui::TextEdit::singleline(line).desired_width(70.0).id(egui::Id::new("intrascript-goto")));
            if !field.has_focus() && !field.lost_focus() {
                field.request_focus();
            }
            if (field.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter))) || ui.button("Go").clicked() {
                go = line.trim().parse::<usize>().ok();
                close = true;
            }
            if ui.small_button("Close").clicked() || ui.input(|i| i.key_pressed(Key::Escape)) {
                close = true;
            }
        });
        if let Some(n) = go {
            self.pending = Some(edit::line_span(&self.text, n.max(1)));
        }
        if close {
            self.goto = None;
        }
    }

    /// The keys the editor answers before the text box sees them: completion, Enter, Tab, Shift+Tab, Ctrl+/,
    /// Ctrl+Space.
    fn edit_keys(&mut self, ui: &Ui) {
        let pressed = |m: Modifiers, k: Key| ui.input_mut(|i| i.consume_key(m, k));
        if let Some((list, picked)) = &mut self.completion {
            let n = list.items.len().max(1);
            if pressed(Modifiers::NONE, Key::ArrowDown) {
                *picked = (*picked + 1) % n;
            }
            if pressed(Modifiers::NONE, Key::ArrowUp) {
                *picked = (*picked + n - 1) % n;
            }
            if pressed(Modifiers::NONE, Key::Escape) {
                self.completion = None;
            } else if pressed(Modifiers::NONE, Key::Enter) || pressed(Modifiers::NONE, Key::Tab) {
                self.accept(None);
            }
            return;
        }
        if pressed(Modifiers::COMMAND, Key::Space) {
            self.open_completion();
        }
        if pressed(Modifiers::SHIFT, Key::Tab) {
            let result = edit::indent(&self.text, self.cursor, true);
            self.apply(result);
        } else if pressed(Modifiers::NONE, Key::Tab) {
            let result = if self.cursor.0 == self.cursor.1 {
                edit::replace(&self.text, self.cursor, edit::INDENT)
            } else {
                edit::indent(&self.text, self.cursor, false)
            };
            self.apply(result);
        }
        if pressed(Modifiers::COMMAND, Key::Slash) {
            let result = edit::toggle_comment(&self.text, self.cursor);
            self.apply(result);
        }
        if pressed(Modifiers::NONE, Key::Enter) {
            let result = edit::newline(&self.text, self.cursor);
            self.apply(result);
        }
    }

    fn open_completion(&mut self) {
        let others: Vec<Block> = self.others.iter().map(|(o, c)| Block { object: *o, code: c.as_str() }).collect();
        let list = complete(&self.text, self.cursor.1, &self.model, &others);
        self.completion = if list.items.is_empty() { None } else { Some((list, 0)) };
    }

    /// Puts the picked completion (or `item`) in place of the word being typed.
    fn accept(&mut self, item: Option<usize>) {
        let Some((list, picked)) = self.completion.take() else { return };
        let Some(c) = list.items.get(item.unwrap_or(picked)) else { return };
        let result = edit::replace(&self.text, (list.start, self.cursor.1), &c.label);
        self.apply(result);
    }

    /// After the text changed by typing: a `.` or `@` opens the completion; while it is open it follows the word.
    fn after_typing(&mut self) {
        let before = self.text.chars().nth(self.cursor.1.saturating_sub(1)).filter(|_| self.cursor.1 > 0);
        match before {
            Some('.' | '@') => self.open_completion(),
            Some(c) if (c.is_alphanumeric() || c == '_') && self.completion.is_some() => self.open_completion(),
            _ => self.completion = None,
        }
    }

    fn text_area(&mut self, ui: &mut Ui) {
        let id = egui::Id::new(EDIT_ID);
        if self.focused {
            self.edit_keys(ui);
        }
        if let Some((a, b)) = self.pending.take() {
            let mut state = TextEditState::load(ui.ctx(), id).unwrap_or_default();
            state.cursor.set_char_range(Some(CCursorRange::two(CCursor::new(a), CCursor::new(b))));
            state.store(ui.ctx(), id);
            ui.memory_mut(|m| m.request_focus(id));
            self.cursor = (a.min(b), a.max(b));
        }
        let error_lines: BTreeSet<u32> = self.diagnostics.iter().filter(|d| d.is_error()).map(|d| d.line).collect();
        let warning_lines: BTreeSet<u32> = self.diagnostics.iter().filter(|d| !d.is_error()).map(|d| d.line).collect();
        let names = Names::of(&self.model);
        let dark = ui.visuals().dark_mode;
        let size = self.font();
        let mut layouter = |ui: &Ui, buf: &dyn egui::TextBuffer, wrap: f32| {
            let mut job = highlight(buf.as_str(), &names, dark, &error_lines, &warning_lines, size);
            job.wrap.max_width = wrap;
            ui.fonts_mut(|f| f.layout_job(job))
        };
        let lines = self.text.lines().count().max(1) + usize::from(self.text.ends_with('\n'));
        let height = (ui.available_height() - 200.0).max(220.0);
        let mut popup_at = None;
        ScrollArea::vertical().id_salt("intrascript-code").max_height(height).show(ui, |ui| {
            ui.horizontal_top(|ui| {
                let numbers: String = (1..=lines).map(|n| format!("{n:>4}\n")).collect();
                ui.add_space(2.0);
                ui.vertical(|ui| {
                    ui.add_space(3.0);
                    ui.label(RichText::new(numbers.trim_end()).font(FontId::monospace(size)).weak());
                });
                let output = egui::TextEdit::multiline(&mut self.text)
                    .id(id)
                    .code_editor()
                    .desired_rows(24)
                    .desired_width(f32::INFINITY)
                    .lock_focus(true)
                    .layouter(&mut layouter)
                    .show(ui);
                self.focused = output.response.has_focus();
                if let Some(r) = output.cursor_range {
                    let (a, b): (usize, usize) = (r.primary.index.into(), r.secondary.index.into());
                    self.cursor = (a.min(b), a.max(b));
                }
                if output.response.changed() {
                    self.after_typing();
                }
                let at = output.galley.pos_from_cursor(CCursor::new(self.cursor.1));
                popup_at = Some(output.galley_pos + at.left_bottom().to_vec2());
            });
        });
        if let Some(pos) = popup_at {
            self.completion_popup(ui.ctx(), pos);
        }
    }

    fn completion_popup(&mut self, ctx: &egui::Context, pos: egui::Pos2) {
        let Some((list, picked)) = &self.completion else { return };
        let mut clicked = None;
        egui::Area::new(egui::Id::new("intrascript-completion")).order(egui::Order::Foreground).fixed_pos(pos).show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_max_width(420.0);
                ScrollArea::vertical().max_height(240.0).show(ui, |ui| {
                    for (k, item) in list.items.iter().enumerate() {
                        let text = RichText::new(format!("{}   {}", item.label, item.kind.tag())).monospace();
                        let r = ui.selectable_label(k == *picked, text).on_hover_text(&item.help);
                        if k == *picked {
                            r.scroll_to_me(None);
                        }
                        if r.clicked() {
                            clicked = Some(k);
                        }
                    }
                });
                ui.label(RichText::new("Up and Down choose · Enter or Tab insert · Esc closes").weak().small());
            });
        });
        if let Some(k) = clicked {
            self.accept(Some(k));
        }
    }

    fn status_line(&self, ui: &mut Ui) {
        let (line, col) = edit::line_col(&self.text, self.cursor.1);
        let lines = self.text.lines().count().max(1);
        let state = if self.text == self.saved { "saved" } else { "not saved" };
        let zoom = (self.font() / FONT_SIZE * 100.0).round();
        ui.label(
            RichText::new(format!(
                "Line {line}, column {col} · {lines} lines · {state} · {zoom}% · Ctrl+Space complete · Ctrl+F find · Ctrl+G line · Ctrl+/ comment"
            ))
            .weak()
            .small(),
        );
    }

    fn problems(&mut self, ui: &mut Ui) {
        let mut go = None;
        ScrollArea::vertical().id_salt("intrascript-problems").max_height(140.0).show(ui, |ui| {
            for d in self.diagnostics() {
                let colour = if d.is_error() { Color32::from_rgb(200, 60, 60) } else { Color32::from_rgb(200, 140, 40) };
                let kind = if d.is_error() { "error" } else { "warning" };
                let text = RichText::new(format!("line {}: {kind} [{}] {}", d.line, d.code, d.message)).color(colour);
                if ui.add(egui::Label::new(text).sense(Sense::click())).on_hover_text("Go to this line").clicked() {
                    go = Some(d.line as usize);
                }
            }
        });
        if let Some(line) = go {
            self.pending = Some(edit::line_span(&self.text, line));
        }
    }

    /// What the code can name: events, the layout's objects with their members, built-ins and units; a click inserts it
    /// at the cursor.
    fn names(&mut self, ui: &mut Ui, app: &mut BimApp) {
        ui.label(RichText::new("Names you can use").strong());
        ui.label(RichText::new("Click one to insert it at the cursor.").weak());
        let mut chosen: Option<String> = None;
        ScrollArea::vertical().id_salt("intrascript-names").max_height(560.0).show(ui, |ui| {
            egui::CollapsingHeader::new("Events").default_open(true).show(ui, |ui| {
                for (name, help) in DECORATORS {
                    if ui.small_button(format!("@{name}")).on_hover_text(help).clicked() {
                        chosen = Some(format!("@{name}"));
                    }
                }
            });
            egui::CollapsingHeader::new(format!("Objects ({})", self.model.objects.len())).default_open(true).show(ui, |ui| {
                for o in &self.model.objects {
                    let title = if o.address.is_empty() || o.address == o.name { o.name.clone() } else { o.address.clone() };
                    egui::CollapsingHeader::new(&title).id_salt(("object", &o.id)).show(ui, |ui| {
                        ui.label(RichText::new(&o.kind).weak());
                        for m in &o.members {
                            let tag = match m.kind {
                                MemberKind::Signal { writable: true } => "signal, write",
                                MemberKind::Signal { writable: false } => "signal",
                                MemberKind::Value => "read",
                                MemberKind::Method => "call",
                            };
                            let text = format!("{}.{}", o.name, m.name);
                            if ui.small_button(&text).on_hover_text(format!("{} ({tag})", m.help)).clicked() {
                                chosen = Some(text);
                            }
                        }
                    });
                }
            });
            egui::CollapsingHeader::new("Load units (pallet.…)").show(ui, |ui| {
                for (name, help) in intracraft_script::host::PALLET_MEMBERS {
                    if ui.small_button(format!("pallet.{name}")).on_hover_text(help).clicked() {
                        chosen = Some(format!("pallet.{name}"));
                    }
                }
            });
            egui::CollapsingHeader::new("Built-ins").show(ui, |ui| {
                for (name, _, help) in intracraft_script::builtins::GLOBALS {
                    if ui.small_button(name).on_hover_text(help).clicked() {
                        chosen = Some(name.to_string());
                    }
                }
                for (name, help) in [
                    ("wait", "wait(2 * s): this handler pauses; the plant goes on"),
                    ("wait_until", "wait_until(condition, timeout=30 * s): True when it held, False at the timeout"),
                ] {
                    if ui.small_button(name).on_hover_text(help).clicked() {
                        chosen = Some(format!("{name}()"));
                    }
                }
            });
            egui::CollapsingHeader::new("Units").show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for (name, ..) in intracraft_script::units::UNITS {
                        if ui.small_button(name).clicked() {
                            chosen = Some(format!(" * {name}"));
                        }
                    }
                });
            });
        });
        if let Some(text) = chosen {
            let result = edit::replace(&self.text, self.cursor, &text);
            self.apply(result);
            app.status = Status { text: format!("Inserted {}", text.trim()), error: false };
        }
    }
}

/// The names the colours tell apart.
struct Names {
    objects: BTreeSet<String>,
    builtins: BTreeSet<&'static str>,
    units: BTreeSet<&'static str>,
}

impl Names {
    fn of(model: &ModelInfo) -> Names {
        let mut objects: BTreeSet<String> = model.objects.iter().map(|o| o.name.clone()).collect();
        // A section's name starts an address: colour it as an object.
        objects.extend(model.objects.iter().filter_map(|o| o.address.split_once('.').map(|(s, _)| s.to_string())));
        Names {
            objects,
            builtins: intracraft_script::builtins::GLOBALS.iter().map(|(n, ..)| *n).chain(["wait", "wait_until"]).collect(),
            units: intracraft_script::units::UNITS.iter().map(|(n, ..)| *n).collect(),
        }
    }
}

/// What a piece of code is, for its colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Part {
    Plain,
    Keyword,
    Event,
    Object,
    Builtin,
    Unit,
    Number,
    Text,
    Comment,
}

/// The code split into coloured parts, line by line: the text and what each part is.
fn parts(code: &str, names: &Names) -> Vec<(String, Part)> {
    let chars: Vec<char> = code.chars().collect();
    let mut out: Vec<(String, Part)> = Vec::new();
    let mut i = 0;
    let push = |out: &mut Vec<(String, Part)>, s: String, p: Part| match out.last_mut() {
        Some((t, q)) if *q == p => t.push_str(&s),
        _ => out.push((s, p)),
    };
    while i < chars.len() {
        let c = chars[i];
        let start = i;
        let part = if c == '#' {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            Part::Comment
        } else if c == '"' || c == '\'' {
            let triple = chars.get(i + 1) == Some(&c) && chars.get(i + 2) == Some(&c);
            i += if triple { 3 } else { 1 };
            while i < chars.len() {
                if chars[i] == '\\' {
                    i += 2;
                    continue;
                }
                if !triple && chars[i] == '\n' {
                    break;
                }
                if chars[i] == c && (!triple || (chars.get(i + 1) == Some(&c) && chars.get(i + 2) == Some(&c))) {
                    i += if triple { 3 } else { 1 };
                    break;
                }
                i += 1;
            }
            Part::Text
        } else if c.is_ascii_digit() {
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '.' || chars[i] == '_') {
                i += 1;
            }
            Part::Number
        } else if c == '@' {
            i += 1;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            Part::Event
        } else if c.is_alphabetic() || c == '_' {
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            let after_dot = start > 0 && chars[start - 1] == '.';
            // A text prefix: f"…", r'…'.
            if matches!(word.as_str(), "f" | "r" | "rf" | "fr") && matches!(chars.get(i), Some('"' | '\'')) {
                Part::Text
            } else if after_dot {
                Part::Plain
            } else if KEYWORDS.contains(&word.as_str()) {
                Part::Keyword
            } else if names.objects.contains(&word) {
                Part::Object
            } else if names.units.contains(word.as_str()) {
                Part::Unit
            } else if names.builtins.contains(word.as_str()) {
                Part::Builtin
            } else {
                Part::Plain
            }
        } else {
            i += 1;
            Part::Plain
        };
        let i_end = i.min(chars.len());
        push(&mut out, chars[start..i_end].iter().collect(), part);
        i = i_end;
    }
    out
}

/// The code as a layout job: coloured parts, problem lines underlined (errors red, warnings amber), at letter size
/// `size`.
fn highlight(code: &str, names: &Names, dark: bool, errors: &BTreeSet<u32>, warnings: &BTreeSet<u32>, size: f32) -> LayoutJob {
    let colour = |p: Part| match (p, dark) {
        (Part::Plain, true) => Color32::from_rgb(220, 220, 220),
        (Part::Plain, false) => Color32::from_rgb(30, 30, 30),
        (Part::Keyword, true) => Color32::from_rgb(198, 120, 221),
        (Part::Keyword, false) => Color32::from_rgb(140, 40, 160),
        (Part::Event, true) => Color32::from_rgb(86, 182, 194),
        (Part::Event, false) => Color32::from_rgb(0, 120, 130),
        (Part::Object, true) => Color32::from_rgb(229, 192, 123),
        (Part::Object, false) => Color32::from_rgb(150, 90, 0),
        (Part::Builtin, true) => Color32::from_rgb(97, 175, 239),
        (Part::Builtin, false) => Color32::from_rgb(20, 90, 190),
        (Part::Unit, true) => Color32::from_rgb(209, 154, 102),
        (Part::Unit, false) => Color32::from_rgb(170, 90, 30),
        (Part::Number, true) => Color32::from_rgb(152, 195, 121),
        (Part::Number, false) => Color32::from_rgb(40, 130, 60),
        (Part::Text, true) => Color32::from_rgb(230, 160, 120),
        (Part::Text, false) => Color32::from_rgb(170, 60, 40),
        (Part::Comment, _) => Color32::from_rgb(120, 130, 120),
    };
    let font = FontId::monospace(size);
    let mut job = LayoutJob::default();
    let mut line = 1u32;
    for (text, part) in parts(code, names) {
        // Split at newlines so each line is underlined on its own.
        for (k, piece) in text.split('\n').enumerate() {
            if k > 0 {
                job.append("\n", 0.0, TextFormat::simple(font.clone(), colour(Part::Plain)));
                line += 1;
            }
            if piece.is_empty() {
                continue;
            }
            let mut format = TextFormat::simple(font.clone(), colour(part));
            if errors.contains(&line) {
                format.underline = Stroke::new(1.5, Color32::from_rgb(220, 50, 50));
            } else if warnings.contains(&line) {
                format.underline = Stroke::new(1.0, Color32::from_rgb(220, 160, 40));
            }
            job.append(piece, 0.0, format);
        }
    }
    job
}

/// The Discrete simulation module's view of the code: an error that stopped it, what it printed, its variables.
pub fn run_panel(ui: &mut Ui, sim: &FlowSimulation, layout: &Layout) {
    let blocks: Vec<&intracraft_library::layout::Instance> =
        layout.instances.iter().filter(|i| i.definition_id == intracraft_script::CODE_BLOCK).collect();
    if blocks.is_empty() {
        return;
    }
    ui.label(RichText::new("Code").strong());
    if let Some(e) = sim.script_error() {
        ui.colored_label(Color32::from_rgb(200, 60, 60), e.to_string());
    }
    if let Some(reason) = sim.stopped() {
        ui.label(format!("The code stopped the run: {reason}"));
    }
    let console = sim.script_console();
    egui::CollapsingHeader::new(format!("Console ({})", console.len())).default_open(true).show(ui, |ui| {
        ScrollArea::vertical().id_salt("intrascript-console").max_height(220.0).stick_to_bottom(true).show(ui, |ui| {
            for l in console.iter().skip(console.len().saturating_sub(CONSOLE_LINES)) {
                let text = format!("{}  {}: {}", intracraft_script::vm::clock(l.time), l.block, l.text);
                if l.warning {
                    ui.colored_label(Color32::from_rgb(200, 140, 40), RichText::new(text).monospace());
                } else {
                    ui.label(RichText::new(text).monospace());
                }
            }
        });
    });
    for b in blocks {
        let vars = sim.script_variables(&b.id);
        egui::CollapsingHeader::new(format!("Variables of {}", intracraft_script::code_name(&b.name))).id_salt(("vars", &b.id)).show(
            ui,
            |ui| {
                if vars.is_empty() {
                    ui.label("None.");
                }
                egui::Grid::new(("vars-grid", &b.id)).num_columns(2).striped(true).show(ui, |ui| {
                    for (name, value) in vars {
                        ui.label(RichText::new(name).monospace());
                        ui.label(RichText::new(value).monospace());
                        ui.end_row();
                    }
                });
            },
        );
    }
    ui.separator();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names() -> Names {
        Names {
            objects: ["Roller3".to_string()].into_iter().collect(),
            builtins: ["print"].into_iter().collect(),
            units: ["s", "mm"].into_iter().collect(),
        }
    }

    #[test]
    fn colours_tell_keywords_events_objects_units_texts_and_comments_apart() {
        let p = parts("@on_enter(Roller3)\ndef f(pallet):  # hold\n    wait(2 * s)\n    print(f\"{x}\", 'a')\n", &names());
        let of = |t: &str| p.iter().find(|(s, _)| s == t).map(|(_, k)| *k);
        assert_eq!(of("@on_enter"), Some(Part::Event));
        assert_eq!(of("Roller3"), Some(Part::Object));
        assert_eq!(of("def"), Some(Part::Keyword));
        assert_eq!(of("# hold"), Some(Part::Comment));
        assert_eq!(of("s"), Some(Part::Unit));
        assert_eq!(of("2"), Some(Part::Number));
        assert!(p.iter().any(|(s, k)| *k == Part::Text && s.contains("'a'")));
        // Nothing is lost: the parts make up the code.
        let joined: String = p.iter().map(|(s, _)| s.as_str()).collect();
        assert!(joined.starts_with("@on_enter(Roller3)\ndef f(pallet):"));
    }

    #[test]
    fn problem_lines_are_underlined_at_the_chosen_size() {
        let errors: BTreeSet<u32> = [2].into_iter().collect();
        let job = highlight("a = 1\nb = c\n", &names(), true, &errors, &BTreeSet::new(), 16.0);
        let underlined: Vec<&str> = job
            .sections
            .iter()
            .filter(|s| s.format.underline.width > 0.0)
            .map(|s| &job.text[s.byte_range.start.0..s.byte_range.end.0])
            .collect();
        assert!(underlined.iter().any(|t| t.contains('b')));
        assert!(!underlined.iter().any(|t| t.contains('a')));
        assert!(job.sections.iter().all(|s| s.format.font_id.size == 16.0));
    }

    #[test]
    fn a_dot_opens_the_members_and_a_pick_replaces_the_word() {
        let model = ModelInfo {
            objects: vec![intracraft_script::ObjectInfo {
                name: "Roller3".into(),
                members: vec![intracraft_script::Member {
                    name: "run".into(),
                    kind: MemberKind::Signal { writable: true },
                    help: "on or off".into(),
                }],
                ..Default::default()
            }],
        };
        let mut e = CodeEditor { model, text: "Roller3.".into(), cursor: (8, 8), ..Default::default() };
        e.after_typing();
        let (list, _) = e.completion.as_ref().unwrap();
        assert_eq!(list.items[0].label, "run");
        e.accept(None);
        assert_eq!(e.text, "Roller3.run");
        assert_eq!(e.pending, Some((11, 11)));
        assert!(e.completion.is_none());
    }

    #[test]
    fn typing_on_closes_the_list_when_the_word_ends() {
        let mut e = CodeEditor { text: "x = ".into(), cursor: (4, 4), ..Default::default() };
        e.after_typing();
        assert!(e.completion.is_none());
    }
}
