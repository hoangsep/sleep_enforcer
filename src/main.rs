use std::process::Command;
use std::time::Duration;

use chrono::{
    DateTime,
    Local,
    NaiveDate,
    TimeDelta,
    TimeZone,
};

use eframe::egui::{
    self,
    Align,
    Color32,
    FontId,
    Layout,
    RichText,
    Vec2,
};

const SHUTDOWN_HOUR: u32 = 0;
const SHUTDOWN_MINUTE: u32 = 5;

struct SleepEnforcer {
    shutdown_at: DateTime<Local>,
    demo_mode: bool,
    shutdown_error: Option<String>,
}

impl SleepEnforcer {
    fn new(demo_mode: bool) -> Self {
        let shutdown_at = if demo_mode {
            Local::now() + TimeDelta::minutes(5)
        } else {
            next_shutdown_time()
        };

        Self {
            shutdown_at,
            demo_mode,
            shutdown_error: None,
        }
    }

    fn remaining_seconds(&self) -> i64 {
        let remaining_ms = self
            .shutdown_at
            .signed_duration_since(Local::now())
            .num_milliseconds()
            .max(0);

        // Ceiling division so 4:59.2 displays as 05:00,
        // rather than 04:59.
        (remaining_ms + 999) / 1000
    }

    fn shutdown_now(&mut self) {
        match Command::new("/usr/bin/systemctl")
            .arg("poweroff")
            .spawn()
        {
            Ok(_) => {
                self.shutdown_error = None;
            }

            Err(error) => {
                self.shutdown_error =
                    Some(format!("Failed to request shutdown: {error}"));
            }
        }
    }
}

impl eframe::App for SleepEnforcer {
    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        _frame: &mut eframe::Frame,
    ) {
        // We don't need to redraw at 60/144 Hz.
        // Update frequently enough that the clock feels responsive.
        ui.ctx()
            .request_repaint_after(Duration::from_millis(200));

        let remaining = self.remaining_seconds();

        // Background.
        ui.painter().rect_filled(
            ui.max_rect(),
            0.0,
            Color32::from_rgb(12, 12, 16),
        );

        ui.allocate_ui_with_layout(
            ui.available_size(),
            Layout::top_down(Align::Center),
            |ui| {
                ui.add_space(ui.available_height() * 0.20);

                ui.label(
                    RichText::new("GO TO BED")
                        .size(60.0)
                        .strong()
                        .color(Color32::WHITE),
                );

                ui.add_space(35.0);

                ui.label(
                    RichText::new(format_duration(remaining))
                        .font(FontId::monospace(150.0))
                        .strong()
                        .color(Color32::from_rgb(255, 90, 90)),
                );

                ui.add_space(30.0);

                if self.demo_mode {
                    ui.label(
                        RichText::new("DEMO MODE — computer will not shut down")
                            .size(24.0)
                            .color(Color32::from_rgb(180, 180, 180)),
                    );
                } else {
                    ui.label(
                        RichText::new(format!(
                            "Computer will shut down at {}",
                            self.shutdown_at.format("%H:%M")
                        ))
                        .size(26.0)
                        .color(Color32::from_rgb(180, 180, 180)),
                    );
                }

                ui.add_space(50.0);

                let button = egui::Button::new(
                    RichText::new("SHUT DOWN NOW")
                        .size(26.0)
                        .strong(),
                )
                .min_size(Vec2::new(320.0, 70.0));

                if self.demo_mode {
                    ui.add_enabled(false, button);
                } else if ui.add(button).clicked() {
                    self.shutdown_now();
                }

                if let Some(error) = &self.shutdown_error {
                    ui.add_space(20.0);

                    ui.label(
                        RichText::new(error)
                            .size(18.0)
                            .color(Color32::from_rgb(255, 100, 100)),
                    );
                }
            },
        );
    }
}

fn local_shutdown_time(date: NaiveDate) -> DateTime<Local> {
    let naive = date
        .and_hms_opt(
            SHUTDOWN_HOUR,
            SHUTDOWN_MINUTE,
            0,
        )
        .expect("invalid shutdown time");

    Local
        .from_local_datetime(&naive)
        .single()
        .expect("shutdown time is ambiguous in local timezone")
}

fn next_shutdown_time() -> DateTime<Local> {
    let now = Local::now();

    let today = now.date_naive();

    let today_shutdown = local_shutdown_time(today);

    if now < today_shutdown {
        return today_shutdown;
    }

    let tomorrow = today
        .succ_opt()
        .expect("date overflow");

    local_shutdown_time(tomorrow)
}

fn format_duration(total_seconds: i64) -> String {
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("{hours:02}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes:02}:{seconds:02}")
    }
}

fn main() -> eframe::Result {
    let demo_mode =
        std::env::args().any(|arg| arg == "--demo");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Sleep Enforcer")
            .with_app_id("sleep-enforcer")
            .with_fullscreen(true)
            .with_always_on_top()
            .with_decorations(false)
            .with_resizable(false)
            .with_close_button(false),

        ..Default::default()
    };

    eframe::run_native(
        "Sleep Enforcer",
        options,
        Box::new(move |_creation_context| {
            Ok(Box::new(
                SleepEnforcer::new(demo_mode)
            ))
        }),
    )
}