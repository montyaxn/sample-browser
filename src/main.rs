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
use rfd::FileDialog;
use rodio::{Decoder, OutputStream, Sink, Source, buffer::SamplesBuffer, source};

fn main() {
    let native_options = eframe::NativeOptions {
        persist_window: true,
        ..Default::default()
    };
    eframe::run_native(
        "Everything Sample Browser",
        native_options,
        Box::new(|cc| {
            Ok(Box::new(BrowserApp::new(cc)))
        }),
    )
    .unwrap();
}

#[derive(Default)]
struct BrowserApp {
    source_path: Option<PathBuf>,
    stream: Option<OutputStream>,
    sink: Option<Sink>,

    search_text: String,
    search_results: Vec<PathBuf>,
    search_index: usize,
    search_should_scroll: bool,
    // playback_pos: Duration,
    // playback_total_duration: Duration,
}

impl BrowserApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        Self::default()
    }

    fn start_playback(&mut self) -> anyhow::Result<()> {
        if let Some(path) = &self.source_path {
            let (stream, stream_handle) = OutputStream::try_default()?;
            self.stream = Some(stream);

            let sink = Sink::try_new(&stream_handle).unwrap();

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

    fn collect_audio_files(directory: &str) -> Vec<PathBuf> {
        let mut audio_files = Vec::new();
        let path = Path::new(directory);

        if path.is_dir() {
            // ディレクトリ内のファイルを列挙
            for entry in fs::read_dir(path).unwrap() {
                let entry = entry.unwrap();
                let entry_path = entry.path();

                // 拡張子が.mp3または.wavのファイルを収集
                if let Some(ext) = entry_path.extension() {
                    if ext == "mp3" || ext == "wav" {
                        audio_files.push(entry_path);
                    }
                }
            }
        }

        audio_files
    }

    // fn playback_pos_slider(&mut self, ui: &mut egui::Ui){
    //     let pos = match &self.sink{
    //         Some(sink) => sink.get_pos().div_duration_f32(self.playback_total_duration),
    //         None => 0.0,
    //     };
    //     let mut pos_slider = pos;
    //     ui.add(egui::Slider::new(&mut pos_slider, 0.0..=1.0));
    //     if pos != pos_slider{

    //     }
    // }
}

impl eframe::App for BrowserApp {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::dark());
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut search_bar = egui::TextEdit::singleline(&mut self.search_text).show(ui);
            if search_bar.response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.search_results =
                    search::search("ext:wav;mp3 ".to_string() + &self.search_text);
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

            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut clicked = false;
                for (i, path) in self.search_results.iter().enumerate() {
                    let response =
                        ui.selectable_label(i == self.search_index, path.to_string_lossy());
                    if i == self.search_index && self.search_should_scroll {
                        response.scroll_to_me(None);
                        self.search_should_scroll = false;
                    }
                    if response.clicked() {
                        self.search_index = i;
                        self.source_path = Some(path.clone());
                        clicked = true;
                    }
                }
                if clicked {
                    self.start_playback();
                };
            });
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
    }
}
