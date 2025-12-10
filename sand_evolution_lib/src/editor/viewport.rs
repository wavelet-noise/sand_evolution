use egui::{Ui, Rect, Color32, Stroke};
use crate::editor::state::EditorState;

pub struct EditorViewport;

impl EditorViewport {
    pub fn ui(ui: &mut Ui, editor_state: &mut EditorState, viewport_rect: Rect) {
        // Handle zoom with mouse wheel
        let response = ui.interact(viewport_rect, ui.id(), egui::Sense::click_and_drag());
        
        if let Some(_hover_pos) = response.hover_pos() {
            // Handle pan with middle mouse button or space+drag
            if response.dragged_by(egui::PointerButton::Middle) {
                let delta = response.drag_delta();
                editor_state.viewport_pan.0 += delta.x;
                editor_state.viewport_pan.1 += delta.y;
            }
        }
        
        // Zoom handling is done in InputHandler
        
        // Draw grid if enabled
        if editor_state.show_grid {
            Self::draw_grid(ui, viewport_rect, editor_state);
        }
        
        // Draw selection rectangle
        if let Some((x1, y1, x2, y2)) = editor_state.selection_rect {
            let rect = Rect::from_min_max(
                egui::pos2(x1.min(x2), y1.min(y2)),
                egui::pos2(x1.max(x2), y1.max(y2)),
            );
            ui.painter().rect_stroke(
                rect,
                0.0,
                Stroke::new(2.0, Color32::from_rgb(100, 150, 255)),
            );
        }
    }
    
    fn draw_grid(ui: &mut Ui, rect: Rect, editor_state: &EditorState) {
        let grid_size = editor_state.grid_size * editor_state.viewport_zoom;
        let pan_x = editor_state.viewport_pan.0;
        let pan_y = editor_state.viewport_pan.1;
        
        let start_x = ((rect.min.x - pan_x) / grid_size).floor() * grid_size + pan_x;
        let start_y = ((rect.min.y - pan_y) / grid_size).floor() * grid_size + pan_y;
        
        let mut x = start_x;
        while x < rect.max.x {
            ui.painter().line_segment(
                [egui::pos2(x, rect.min.y), egui::pos2(x, rect.max.y)],
                Stroke::new(1.0, Color32::from_rgba_unmultiplied(100, 100, 100, 50)),
            );
            x += grid_size;
        }
        
        let mut y = start_y;
        while y < rect.max.y {
            ui.painter().line_segment(
                [egui::pos2(rect.min.x, y), egui::pos2(rect.max.x, y)],
                Stroke::new(1.0, Color32::from_rgba_unmultiplied(100, 100, 100, 50)),
            );
            y += grid_size;
        }
    }
    
    pub fn status_bar(ui: &mut Ui, editor_state: &EditorState, cursor_pos: Option<(f32, f32)>) {
        ui.horizontal(|ui| {
            ui.label(format!("Zoom: {:.1}%", editor_state.viewport_zoom * 100.0));
            ui.separator();
            
            if let Some((x, y)) = cursor_pos {
                let snapped = editor_state.snap_position(x, y);
                ui.label(format!("Cursor: ({:.1}, {:.1})", x, y));
                if editor_state.snap_to_grid {
                    ui.label(format!("Snapped: ({:.1}, {:.1})", snapped.0, snapped.1));
                }
            }
            
            ui.separator();
            
            ui.label(format!("Selected: {} object(s)", editor_state.selected_entities.len()));
            
            ui.separator();
            
            let mode_text = match editor_state.mode {
                crate::editor::state::EditorMode::Select => "Select",
                crate::editor::state::EditorMode::Move => "Move",
                crate::editor::state::EditorMode::Rotate => "Rotate",
                crate::editor::state::EditorMode::Scale => "Scale",
            };
            ui.label(format!("Mode: {}", mode_text));
        });
    }
    
    pub fn frame_selected(editor_state: &mut EditorState, _world: &specs::World) {
        // TODO: Calculate bounding box of selected entities and adjust viewport
        editor_state.add_toast("Frame selected - not yet implemented".to_string(), 
                               crate::editor::state::ToastLevel::Info);
    }
}
