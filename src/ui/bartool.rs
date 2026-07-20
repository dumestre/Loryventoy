use eframe::egui::{self, Color32, Rect, Sense, Ui, Vec2};

pub struct BarTool {
    pub is_playing: bool,
    pub loop_enabled: bool,
    pub request_prev_frame: bool,
    pub request_next_frame: bool,
    pub request_stop: bool,
}

impl BarTool {
    pub fn new() -> Self {
        Self {
            is_playing: false,
            loop_enabled: false,
            request_prev_frame: false,
            request_next_frame: false,
            request_stop: false,
        }
    }

    fn button(ui: &mut Ui, icon: &egui::Image) -> bool {
        ui.add(
            icon.clone()
                .max_size(Vec2::new(20.0, 20.0))
                .sense(Sense::click()),
        )
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .clicked()
    }

    pub fn show(&mut self, ui: &mut Ui, preview_rect: Rect) {
        self.request_prev_frame = false;
        self.request_next_frame = false;
        self.request_stop = false;

        let padding = 12.0;
        let bar_height = 30.0;
        let bar_fill = Color32::from_rgb(60, 60, 78);
        let playback_width = 220.0;
        let playback_pos = egui::pos2(
            preview_rect.center().x - playback_width / 2.0,
            preview_rect.max.y - bar_height - padding,
        );

        egui::Area::new(egui::Id::new("floating_bar_playback"))
            .fixed_pos(playback_pos)
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                egui::Frame::new()
                    .fill(bar_fill)
                    .stroke(egui::Stroke::new(1.0, Color32::from_gray(75)))
                    .corner_radius(10.0)
                    .inner_margin(egui::Margin::symmetric(8, 4))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 4.0;

                            if Self::button(ui, &egui::Image::new(egui::include_image!("icons/anterior.svg")))
                            {
                                self.request_prev_frame = true;
                            }

                            let play_img = if self.is_playing {
                                egui::Image::new(egui::include_image!("icons/pause.svg"))
                            } else {
                                egui::Image::new(egui::include_image!("icons/play.svg"))
                            };
                            if Self::button(ui, &play_img) {
                                self.is_playing = !self.is_playing;
                            }

                            if Self::button(ui, &egui::Image::new(egui::include_image!("icons/stop.svg")))
                            {
                                self.request_stop = true;
                            }

                            if Self::button(ui, &egui::Image::new(egui::include_image!("icons/proximo.svg")))
                            {
                                self.request_next_frame = true;
                            }

                            ui.add(egui::Separator::default().vertical().spacing(6.0));

                            let loop_img = egui::Image::new(egui::include_image!("icons/loop.svg"))
                                .tint(if self.loop_enabled {
                                    Color32::from_rgb(140, 160, 255)
                                } else {
                                    Color32::WHITE
                                });
                            if Self::button(ui, &loop_img) {
                                self.loop_enabled = !self.loop_enabled;
                            }
                        });
                    });
            });
    }
}
