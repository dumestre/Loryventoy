use eframe::egui::{self, Color32, Rect, Sense, Ui, Vec2};

pub struct BarTool {
    pub is_playing: bool,
    pub loop_enabled: bool,
}

impl BarTool {
    pub fn new() -> Self {
        Self {
            is_playing: false,
            loop_enabled: false,
        }
    }

    pub fn show(&mut self, ui: &mut Ui, preview_rect: Rect) {
        let padding = 12.0;
        let bar_height = 30.0;
        let bar_fill = Color32::from_rgb(60, 60, 78);
        let icon_size = Vec2::new(20.0, 20.0);

        // --- Barra de playback: FUNDO do preview ---
        let playback_width = 220.0;
        let playback_pos = egui::pos2(
            preview_rect.center().x - playback_width / 2.0,
            preview_rect.max.y - bar_height - padding,
        );

        egui::Area::new(egui::Id::new("floating_bar_playback"))
            .fixed_pos(playback_pos)
            .order(egui::Order::Middle)
            .show(ui.ctx(), |ui| {
                egui::Frame::new()
                    .fill(bar_fill)
                    .stroke(egui::Stroke::new(1.0, Color32::from_gray(75)))
                    .corner_radius(10.0)
                    .inner_margin(egui::Margin::symmetric(8, 4))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 4.0;

                            // Anterior
                            if ui.add(
                                egui::Image::new(egui::include_image!("icons/anterior.svg"))
                                    .max_size(icon_size)
                                    .sense(Sense::click())
                            ).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {}

                            // Play/Pause
                            let play_img = if self.is_playing {
                                egui::Image::new(egui::include_image!("icons/pause.svg"))
                                    .max_size(icon_size)
                                    .sense(Sense::click())
                            } else {
                                egui::Image::new(egui::include_image!("icons/play.svg"))
                                    .max_size(icon_size)
                                    .sense(Sense::click())
                            };
                            if ui.add(play_img)
                                .on_hover_cursor(egui::CursorIcon::PointingHand)
                                .clicked()
                            {
                                self.is_playing = !self.is_playing;
                            }

                            // Stop
                            if ui.add(
                                egui::Image::new(egui::include_image!("icons/stop.svg"))
                                    .max_size(icon_size)
                                    .sense(Sense::click())
                            ).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                self.is_playing = false;
                            }

                            // Proximo
                            if ui.add(
                                egui::Image::new(egui::include_image!("icons/proximo.svg"))
                                    .max_size(icon_size)
                                    .sense(Sense::click())
                            ).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {}

                            // Separador
                            ui.add(egui::Separator::default().vertical().spacing(6.0));

                            // Loop — destaque quando ativo
                            let loop_resp = ui.add(
                                egui::Image::new(egui::include_image!("icons/loop.svg"))
                                    .max_size(icon_size)
                                    .sense(Sense::click())
                                    .tint(if self.loop_enabled {
                                        Color32::from_rgb(140, 160, 255)
                                    } else {
                                        Color32::WHITE
                                    })
                            ).on_hover_cursor(egui::CursorIcon::PointingHand);

                            if loop_resp.clicked() {
                                self.loop_enabled = !self.loop_enabled;
                            }
                        });
                    });
            });
    }
}
