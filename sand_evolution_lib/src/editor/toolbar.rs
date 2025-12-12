use crate::editor::state::{EditorMode, EditorState};
use crate::editor::undo_redo::UndoRedo;
use egui::{Button, RichText};

pub struct EditorToolbar;

impl EditorToolbar {
    pub fn ui(ui: &mut egui::Ui, editor_state: &mut EditorState, undo_redo: &UndoRedo) {
        ui.horizontal(|ui| {
            ui.set_height(32.0);
            ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);

            // Undo/Redo buttons
            let undo_enabled = undo_redo.can_undo();
            let undo_text = if let Some(name) = undo_redo.undo_name() {
                format!("↶ Undo: {}", name)
            } else {
                "↶ Undo".to_string()
            };
            if ui
                .add_enabled(
                    undo_enabled,
                    Button::new(RichText::new(&undo_text).size(12.0)),
                )
                .clicked()
            {
                // Will be handled by keyboard handler
            }

            let redo_enabled = undo_redo.can_redo();
            let redo_text = if let Some(name) = undo_redo.redo_name() {
                format!("↷ Redo: {}", name)
            } else {
                "↷ Redo".to_string()
            };
            if ui
                .add_enabled(
                    redo_enabled,
                    Button::new(RichText::new(&redo_text).size(12.0)),
                )
                .clicked()
            {
                // Will be handled by keyboard handler
            }

            ui.separator();

            // Select tool
            let select_selected = editor_state.mode == EditorMode::Select;
            let mut select_btn = Button::new(RichText::new("Select").size(12.0));
            if select_selected {
                select_btn =
                    select_btn.fill(egui::Color32::from_rgba_unmultiplied(100, 150, 255, 100));
            }
            if ui
                .add(select_btn)
                .on_hover_text("Select tool (Q)")
                .clicked()
            {
                editor_state.mode = EditorMode::Select;
            }

            // Move tool
            let move_selected = editor_state.mode == EditorMode::Move;
            let mut move_btn = Button::new(RichText::new("Move").size(12.0));
            if move_selected {
                move_btn = move_btn.fill(egui::Color32::from_rgba_unmultiplied(100, 150, 255, 100));
            }
            if ui.add(move_btn).on_hover_text("Move tool (G)").clicked() {
                editor_state.mode = EditorMode::Move;
            }

            // Rotate tool
            let rotate_selected = editor_state.mode == EditorMode::Rotate;
            let mut rotate_btn = Button::new(RichText::new("Rotate").size(12.0));
            if rotate_selected {
                rotate_btn =
                    rotate_btn.fill(egui::Color32::from_rgba_unmultiplied(100, 150, 255, 100));
            }
            if ui
                .add(rotate_btn)
                .on_hover_text("Rotate tool (R)")
                .clicked()
            {
                editor_state.mode = EditorMode::Rotate;
            }

            // Scale tool
            let scale_selected = editor_state.mode == EditorMode::Scale;
            let mut scale_btn = Button::new(RichText::new("Scale").size(12.0));
            if scale_selected {
                scale_btn =
                    scale_btn.fill(egui::Color32::from_rgba_unmultiplied(100, 150, 255, 100));
            }
            if ui.add(scale_btn).on_hover_text("Scale tool (S)").clicked() {
                editor_state.mode = EditorMode::Scale;
            }

            ui.separator();

            // Grid toggle
            ui.checkbox(&mut editor_state.show_grid, "Grid")
                .on_hover_text("Toggle grid visibility");

            // Snap toggle
            ui.checkbox(&mut editor_state.snap_to_grid, "Snap")
                .on_hover_text("Snap to grid");
        });
    }

    pub fn handle_keyboard(editor_state: &mut EditorState, key: &egui::Key) -> bool {
        match key {
            egui::Key::Q => {
                editor_state.mode = EditorMode::Select;
                true
            }
            egui::Key::G => {
                editor_state.mode = EditorMode::Move;
                true
            }
            egui::Key::R => {
                editor_state.mode = EditorMode::Rotate;
                true
            }
            egui::Key::S => {
                editor_state.mode = EditorMode::Scale;
                true
            }
            _ => false,
        }
    }
}
