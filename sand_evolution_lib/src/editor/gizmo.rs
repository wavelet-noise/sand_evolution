use crate::ecs::components::Position;
use crate::editor::state::{EditorMode, EditorState, GizmoHandle};
use egui::{Color32, Painter, Pos2, Stroke};
use specs::{World, WorldExt};

pub struct GizmoSystem;

impl GizmoSystem {
    pub fn draw(&self, painter: &Painter, editor_state: &EditorState, world: &World) {
        if editor_state.selected_entities.is_empty() {
            return;
        }

        let positions = world.read_storage::<Position>();

        for entity in &editor_state.selected_entities {
            if let Some(pos) = positions.get(*entity) {
                Self::draw_gizmo_for_position(
                    painter,
                    Pos2::new(pos.x, pos.y),
                    editor_state.mode,
                    editor_state.gizmo_handle,
                );
            }
        }
    }

    fn draw_gizmo_for_position(
        painter: &Painter,
        pos: Pos2,
        mode: EditorMode,
        active_handle: GizmoHandle,
    ) {
        match mode {
            EditorMode::Select => {
                // Draw selection indicator
                painter.circle_filled(pos, 5.0, Color32::from_rgb(100, 150, 255));
            }
            EditorMode::Move => {
                Self::draw_move_gizmo(painter, pos, active_handle);
            }
            EditorMode::Rotate => {
                Self::draw_rotate_gizmo(painter, pos, active_handle);
            }
            EditorMode::Scale => {
                Self::draw_scale_gizmo(painter, pos, active_handle);
            }
        }
    }

    fn draw_move_gizmo(painter: &Painter, pos: Pos2, active_handle: GizmoHandle) {
        let size = 20.0;

        // X axis (red)
        let x_color = if active_handle == GizmoHandle::MoveX || active_handle == GizmoHandle::MoveXY
        {
            Color32::from_rgb(255, 200, 100)
        } else {
            Color32::from_rgb(255, 50, 50)
        };
        painter.line_segment(
            [pos, Pos2::new(pos.x + size, pos.y)],
            Stroke::new(2.0, x_color),
        );
        painter.circle_filled(Pos2::new(pos.x + size, pos.y), 4.0, x_color);

        // Y axis (green)
        let y_color = if active_handle == GizmoHandle::MoveY || active_handle == GizmoHandle::MoveXY
        {
            Color32::from_rgb(200, 255, 100)
        } else {
            Color32::from_rgb(50, 255, 50)
        };
        painter.line_segment(
            [pos, Pos2::new(pos.x, pos.y + size)],
            Stroke::new(2.0, y_color),
        );
        painter.circle_filled(Pos2::new(pos.x, pos.y + size), 4.0, y_color);

        // Center handle (yellow)
        let center_color = if active_handle == GizmoHandle::MoveXY {
            Color32::from_rgb(255, 255, 100)
        } else {
            Color32::from_rgb(200, 200, 50)
        };
        painter.circle_filled(pos, 6.0, center_color);
    }

    fn draw_rotate_gizmo(painter: &Painter, pos: Pos2, active_handle: GizmoHandle) {
        let radius = 30.0;
        let is_active = active_handle == GizmoHandle::Rotate;

        let color = if is_active {
            Color32::from_rgb(100, 200, 255)
        } else {
            Color32::from_rgb(50, 150, 255)
        };

        // Draw circle
        painter.circle_stroke(pos, radius, Stroke::new(2.0, color));

        // Draw handle
        let handle_pos = Pos2::new(pos.x + radius, pos.y);
        painter.circle_filled(handle_pos, 5.0, color);
    }

    fn draw_scale_gizmo(painter: &Painter, pos: Pos2, active_handle: GizmoHandle) {
        let size = 20.0;

        // X axis
        let x_color =
            if active_handle == GizmoHandle::ScaleX || active_handle == GizmoHandle::ScaleXY {
                Color32::from_rgb(255, 200, 100)
            } else {
                Color32::from_rgb(255, 50, 50)
            };
        painter.line_segment(
            [pos, Pos2::new(pos.x + size, pos.y)],
            Stroke::new(2.0, x_color),
        );
        painter.rect_filled(
            egui::Rect::from_center_size(Pos2::new(pos.x + size, pos.y), egui::vec2(6.0, 6.0)),
            0.0,
            x_color,
        );

        // Y axis
        let y_color =
            if active_handle == GizmoHandle::ScaleY || active_handle == GizmoHandle::ScaleXY {
                Color32::from_rgb(200, 255, 100)
            } else {
                Color32::from_rgb(50, 255, 50)
            };
        painter.line_segment(
            [pos, Pos2::new(pos.x, pos.y + size)],
            Stroke::new(2.0, y_color),
        );
        painter.rect_filled(
            egui::Rect::from_center_size(Pos2::new(pos.x, pos.y + size), egui::vec2(6.0, 6.0)),
            0.0,
            y_color,
        );

        // Center handle
        let center_color = if active_handle == GizmoHandle::ScaleXY {
            Color32::from_rgb(255, 255, 100)
        } else {
            Color32::from_rgb(200, 200, 50)
        };
        painter.rect_filled(
            egui::Rect::from_center_size(pos, egui::vec2(8.0, 8.0)),
            0.0,
            center_color,
        );
    }

    pub fn hit_test(
        &self,
        pos: Pos2,
        entity_pos: Pos2,
        mode: EditorMode,
        viewport_zoom: f32,
    ) -> GizmoHandle {
        let threshold = 8.0 / viewport_zoom;
        let size = 20.0;

        match mode {
            EditorMode::Move => {
                let dist_to_center = pos.distance(entity_pos);
                if dist_to_center < threshold {
                    return GizmoHandle::MoveXY;
                }

                let dist_to_x = pos.distance(Pos2::new(entity_pos.x + size, entity_pos.y));
                if dist_to_x < threshold {
                    return GizmoHandle::MoveX;
                }

                let dist_to_y = pos.distance(Pos2::new(entity_pos.x, entity_pos.y + size));
                if dist_to_y < threshold {
                    return GizmoHandle::MoveY;
                }
            }
            EditorMode::Rotate => {
                let radius = 30.0;
                let handle_pos = Pos2::new(entity_pos.x + radius, entity_pos.y);
                if pos.distance(handle_pos) < threshold {
                    return GizmoHandle::Rotate;
                }
            }
            EditorMode::Scale => {
                let dist_to_center = pos.distance(entity_pos);
                if dist_to_center < threshold {
                    return GizmoHandle::ScaleXY;
                }

                let dist_to_x = pos.distance(Pos2::new(entity_pos.x + size, entity_pos.y));
                if dist_to_x < threshold {
                    return GizmoHandle::ScaleX;
                }

                let dist_to_y = pos.distance(Pos2::new(entity_pos.x, entity_pos.y + size));
                if dist_to_y < threshold {
                    return GizmoHandle::ScaleY;
                }
            }
            _ => {}
        }

        GizmoHandle::None
    }
}
