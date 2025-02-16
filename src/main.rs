mod search;

use std::{
    fs::{self, File},
    io::BufReader,
    path::{Path, PathBuf},
    time::Duration,
};

use eframe::egui::{
    self, Key,
    text::{CCursor, CCursorRange},
};
use egui::{Align, Margin, Pos2, Rect, RichText, Sense, Ui};
use rfd::FileDialog;
use rodio::{
    Decoder, OutputStream, OutputStreamHandle, Sink, Source, buffer::SamplesBuffer, source,
};

fn main() {
    let native_options = eframe::NativeOptions {
        persist_window: true,
        ..Default::default()
    };
    eframe::run_native(
        "Everything Sample Browser",
        native_options,
        Box::new(|cc| Ok(Box::new(BrowserApp::new(cc)))),
    )
    .unwrap();
}

struct BrowserApp {
    source_path: Option<PathBuf>,
    stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sink: Option<Sink>,
    volume: f32,

    search_text: String,
    search_results: Vec<PathBuf>,
    search_index: usize,
    search_should_scroll: bool,
    search_row_height: f32,
    search_scroll_offset: f32,
    // playback_pos: Duration,
    // playback_total_duration: Duration,
}

impl BrowserApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        Self {
            volume: 0.7,
            stream,
            stream_handle,
            source_path: Default::default(),
            sink: Default::default(),
            search_text: Default::default(),
            search_results: Default::default(),
            search_index: Default::default(),
            search_should_scroll: Default::default(),
            search_row_height: 0.0,
            search_scroll_offset:0.0,
        }
    }

    fn start_playback(&mut self) -> anyhow::Result<()> {
        if let Some(path) = &self.source_path {
            let sink = Sink::try_new(&self.stream_handle).unwrap();
            sink.set_volume(self.volume);

            let file = File::open(path)?;
            let data = BufReader::new(file);
            let source = Decoder::new(data)?;

            sink.append(source);
            self.sink = Some(sink);
        }
        Ok(())
    }

    fn toggle_playback(&mut self) {
        if let Some(sink) = &self.sink {
            if sink.is_paused() {
                sink.play();
            } else {
                sink.pause();
            }
        }
    }

    fn make_scroll_area(&mut self, ui: &mut Ui) {
        let text_style = egui::TextStyle::Body;
        let row_height = ui.text_style_height(&text_style);
        if self.search_should_scroll {
            let spacing_y = ui.spacing().item_spacing.y;
            let area_offset = ui.cursor();
            let y = area_offset.top() + self.search_index as f32 * (row_height + spacing_y);
            let target_rect = Rect {
                min: Pos2 {
                    x: 0.0,
                    y: y - self.search_scroll_offset,
                },
                max: Pos2 {
                    x: 10.0,
                    y: y + row_height - self.search_scroll_offset,
                },
            };
            // println!("{}",target_rect);
            ui.scroll_to_rect(target_rect, None);
            self.search_should_scroll = false;
        }
        let scroll = egui::ScrollArea::vertical().auto_shrink([false;2]).show_rows(
            ui,
            row_height,
            self.search_results.len(),
            |ui, row_range| {
                let mut clicked = false;
                for i in row_range {
                    let mut frame = egui::Frame::default()
                        .inner_margin(Margin {
                            left: 4,
                            right: 4,
                            top: 1,
                            bottom: 1,
                        })
                        .corner_radius(2)
                        .begin(ui);

                    {
                        let response = frame.content_ui.add(
                            egui::Label::new(if i == self.search_index {
                                RichText::new(self.search_results[i].to_string_lossy())
                                    .color(egui::Color32::WHITE)
                            } else {
                                RichText::new(self.search_results[i].to_string_lossy())
                            })
                            .truncate()
                            .selectable(false)
                            .sense(Sense::click()),
                        );
                        if i == self.search_index {
                            frame.frame.fill = egui::Color32::DARK_GRAY;
                        }
                        if response.clicked() {
                            self.search_index = i;
                            self.source_path = Some(self.search_results[i].clone());
                            clicked = true;
                        }
                    }

                    let frame_res = frame.end(ui);
                    if i == 0 {
                        self.search_row_height = frame_res.rect.height();
                        // println!("{}",self.search_row_height);
                    }
                }
                if clicked {
                    self.start_playback();
                };
            },
        );
        self.search_scroll_offset = scroll.state.offset.y;
        // println!("{}",self.search_scroll_offset);
    }
}

impl eframe::App for BrowserApp {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::dark());
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut search_bar = egui::TextEdit::singleline(&mut self.search_text)
                .hint_text("Type your query")
                .show(ui);
            if search_bar.response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.search_results = search::search(
                    "ext:wav;mp3 path: !__MACOS !RECYCLE.BIN ".to_string() + &self.search_text,
                );
                self.search_index = 0;
            }
            if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(Key::F)) {
                search_bar.response.request_focus();
                search_bar
                    .state
                    .cursor
                    .set_char_range(Some(CCursorRange::two(
                        CCursor::new(0),
                        CCursor::new(self.search_text.len()),
                    )));

                search_bar.state.store(ui.ctx(), search_bar.response.id);
            }

            if ui
                .add(egui::Slider::new(&mut self.volume, 0.0..=1.0).text("volume"))
                .changed()
            {
                if let Some(sink) = &self.sink {
                    sink.set_volume(self.volume);
                }
            }

            self.make_scroll_area(ui);
        });

        if ctx.input(|i| i.key_pressed(Key::Space)) {
            self.toggle_playback();
        }

        if ctx.input(|i| i.key_pressed(Key::ArrowUp)) && !self.search_results.is_empty() {
            if self.search_index > 0 {
                self.search_index -= 1;
            }
            self.source_path = Some(self.search_results[self.search_index].clone());
            self.search_should_scroll = true;
            self.start_playback();
        }

        if ctx.input(|i| i.key_pressed(Key::ArrowDown)) && !self.search_results.is_empty() {
            if self.search_index < self.search_results.len() - 1 {
                self.search_index += 1;
            }
            self.source_path = Some(self.search_results[self.search_index].clone());
            self.search_should_scroll = true;
            self.start_playback();
        }
        if ctx.input(|i| i.key_pressed(Key::ArrowRight)) && !self.search_results.is_empty() {
            self.start_playback();
        }
    }
}
