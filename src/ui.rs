//! UI helpers.

/// Show an alert message.
pub fn alert(msg: impl ToString) {
    let msg = msg.to_string();

    let mut native_options = eframe::NativeOptions::default();
    native_options.always_on_top = true;
    native_options.initial_window_size = Some(egui::Vec2::new(400.0, 150.0));
    native_options.resizable = false;
    native_options.centered = true;

    struct AlertApp {
        msg: String,
    }

    impl eframe::App for AlertApp {
        fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.horizontal_top(|ui| {
                    ui.label(egui::RichText::new("⚠").size(48.0));
                    ui.add_sized([ui.available_width(), 0.0], egui::Label::new(&self.msg).wrap(true));
                });

                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    if ui.button("OK").clicked() {
                        frame.close();
                    }
                });
            });
        }
    }
    
    eframe::run_native(
        "Alert",
        native_options,
        Box::new(|_cc| Box::new(AlertApp { msg })),
    );
}
