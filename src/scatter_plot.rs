use eframe::egui;

use crate::trajectory::EeTrajectory;

/// One episode's furthest reach point in the x-y plane.
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub(crate) struct FurthestReachPoint {
    pub episode_index: usize,
    pub position: [f64; 3],
    pub distance_xy: f64,
}

/// Find the frame with maximum x-y distance from the first frame.
/// Z is ignored deliberately — furthest in 3D is not necessarily furthest in 2D.
pub(crate) fn compute_furthest_reach(
    traj: &EeTrajectory,
    episode_index: usize,
) -> Option<FurthestReachPoint> {
    let positions = &traj.positions;
    let start = *positions.first()?;
    let mut best_idx = 0;
    let mut best_d2 = 0.0f64;
    for (i, p) in positions.iter().enumerate() {
        let dx = p[0] - start[0];
        let dy = p[1] - start[1];
        let d2 = dx * dx + dy * dy;
        if d2 > best_d2 {
            best_d2 = d2;
            best_idx = i;
        }
    }
    Some(FurthestReachPoint {
        episode_index,
        position: positions[best_idx],
        distance_xy: best_d2.sqrt(),
    })
}

/// Render a 2D scatter of per-episode furthest reach points.
pub(crate) fn show_scatter_plot(
    ui: &mut egui::Ui,
    points: &[FurthestReachPoint],
    _accent: egui::Color32,
) {
    let dot_color = egui::Color32::from_rgb(255, 60, 60);
    let available = ui.available_size();
    let (response, painter) = ui.allocate_painter(available, egui::Sense::hover());
    let rect = response.rect;

    painter.rect_filled(rect, 4.0, egui::Color32::from_gray(20));

    if points.is_empty() {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "No data",
            egui::FontId::monospace(11.0),
            egui::Color32::from_gray(120),
        );
        return;
    }

    let mut raw_min_x = f64::MAX;
    let mut raw_max_x = f64::MIN;
    let mut raw_min_y = f64::MAX;
    let mut raw_max_y = f64::MIN;
    for p in points {
        raw_min_x = raw_min_x.min(p.position[0]);
        raw_max_x = raw_max_x.max(p.position[0]);
        raw_min_y = raw_min_y.min(p.position[1]);
        raw_max_y = raw_max_y.max(p.position[1]);
    }

    let pad_x = ((raw_max_x - raw_min_x) * 0.05).max(0.01);
    let pad_y = ((raw_max_y - raw_min_y) * 0.05).max(0.01);
    let mut min_x = raw_min_x - pad_x;
    let mut max_x = raw_max_x + pad_x;
    let mut min_y = raw_min_y - pad_y;
    let mut max_y = raw_max_y + pad_y;

    // Equal aspect so the workspace isn't visually distorted.
    let span = (max_x - min_x).max(max_y - min_y);
    let cx = (min_x + max_x) * 0.5;
    let cy = (min_y + max_y) * 0.5;
    min_x = cx - span * 0.5;
    max_x = cx + span * 0.5;
    min_y = cy - span * 0.5;
    max_y = cy + span * 0.5;

    let plot_rect = rect.shrink(20.0);
    let proj = |wx: f64, wy: f64| -> egui::Pos2 {
        let nx = ((wx - min_x) / (max_x - min_x)) as f32;
        let ny = ((wy - min_y) / (max_y - min_y)) as f32;
        egui::pos2(
            plot_rect.min.x + nx * plot_rect.width(),
            plot_rect.max.y - ny * plot_rect.height(),
        )
    };

    let grid_color = egui::Color32::from_rgb(20, 75, 82);
    let axis_color = egui::Color32::from_gray(80);
    let step = nice_grid_step(span);

    let mut x = (min_x / step).floor() * step;
    let x_end = (max_x / step).ceil() * step;
    while x <= x_end + step * 0.01 {
        painter.line_segment(
            [proj(x, min_y), proj(x, max_y)],
            egui::Stroke::new(1.0, grid_color),
        );
        x += step;
    }
    let mut y = (min_y / step).floor() * step;
    let y_end = (max_y / step).ceil() * step;
    while y <= y_end + step * 0.01 {
        painter.line_segment(
            [proj(min_x, y), proj(max_x, y)],
            egui::Stroke::new(1.0, grid_color),
        );
        y += step;
    }

    if min_x <= 0.0 && max_x >= 0.0 {
        painter.line_segment(
            [proj(0.0, min_y), proj(0.0, max_y)],
            egui::Stroke::new(1.0, axis_color),
        );
    }
    if min_y <= 0.0 && max_y >= 0.0 {
        painter.line_segment(
            [proj(min_x, 0.0), proj(max_x, 0.0)],
            egui::Stroke::new(1.0, axis_color),
        );
    }

    for p in points {
        let pos = proj(p.position[0], p.position[1]);
        painter.circle_filled(pos, 3.0, dot_color);
    }

    let label_color = egui::Color32::from_gray(150);
    painter.text(
        egui::pos2(rect.max.x - 6.0, rect.max.y - 4.0),
        egui::Align2::RIGHT_BOTTOM,
        "X →",
        egui::FontId::monospace(10.0),
        label_color,
    );
    painter.text(
        egui::pos2(rect.min.x + 4.0, rect.min.y + 4.0),
        egui::Align2::LEFT_TOP,
        "↑ Y",
        egui::FontId::monospace(10.0),
        label_color,
    );
    painter.text(
        egui::pos2(rect.max.x - 6.0, rect.min.y + 4.0),
        egui::Align2::RIGHT_TOP,
        format!("{} episodes", points.len()),
        egui::FontId::monospace(10.0),
        label_color,
    );
    let span_text = format!(
        "reach {:.0}×{:.0}mm  grid {:.0}mm",
        (raw_max_x - raw_min_x) * 1000.0,
        (raw_max_y - raw_min_y) * 1000.0,
        step * 1000.0,
    );
    painter.text(
        egui::pos2(rect.min.x + 4.0, rect.max.y - 4.0),
        egui::Align2::LEFT_BOTTOM,
        &span_text,
        egui::FontId::monospace(10.0),
        label_color,
    );
}

fn nice_grid_step(span: f64) -> f64 {
    if span <= 0.0 {
        return 0.01;
    }
    let raw = span / 6.0;
    let mag = 10.0_f64.powf(raw.log10().floor());
    let norm = raw / mag;
    let step = if norm < 1.5 {
        1.0
    } else if norm < 3.5 {
        2.0
    } else if norm < 7.5 {
        5.0
    } else {
        10.0
    };
    step * mag
}
