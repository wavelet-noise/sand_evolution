use egui::{Context, Pos2};
use specs::{World, WorldExt};
use crate::editor::state::{EditorState, EditorMode};
use crate::editor::gizmo::GizmoSystem;
use crate::ecs::components::Position;

pub struct InputHandler;

impl InputHandler {
    pub fn handle_mouse_input(
        ctx: &Context,
        editor_state: &mut EditorState,
        world: &mut World,
        undo_redo: &mut crate::editor::undo_redo::UndoRedo,
    ) {
        let gizmo = GizmoSystem;
        
        // Get mouse position in world coordinates
        if let Some(pointer_pos) = ctx.pointer_latest_pos() {
            let world_pos = Self::screen_to_world(pointer_pos, editor_state);
            
            // Handle left mouse button - simplified for now
            // TODO: Proper input handling with egui 0.19 API
            let is_primary_down = false; // Will be handled differently
            if is_primary_down {
                if !editor_state.is_dragging {
                    editor_state.is_dragging = true;
                    editor_state.drag_start_pos = (world_pos.x, world_pos.y);
                    editor_state.drag_current_pos = (world_pos.x, world_pos.y);
                    
                    // Check gizmo hit test
                    if !editor_state.selected_entities.is_empty() {
                        {
                            let positions = world.read_storage::<Position>();
                            for entity in &editor_state.selected_entities {
                                if let Some(pos) = positions.get(*entity) {
                                    let entity_pos = Pos2::new(pos.x, pos.y);
                                    editor_state.gizmo_handle = gizmo.hit_test(
                                        world_pos,
                                        entity_pos,
                                        editor_state.mode,
                                        editor_state.viewport_zoom,
                                    );
                                    if editor_state.gizmo_handle != crate::editor::state::GizmoHandle::None {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    
                    // If not clicking gizmo, handle selection
                    if editor_state.gizmo_handle == crate::editor::state::GizmoHandle::None {
                        match editor_state.mode {
                            EditorMode::Select => {
                                // Start selection rectangle
                                editor_state.selection_rect = Some((
                                    world_pos.x,
                                    world_pos.y,
                                    world_pos.x,
                                    world_pos.y,
                                ));
                            }
                            _ => {}
                        }
                    }
                } else {
                    // Update drag
                    editor_state.drag_current_pos = (world_pos.x, world_pos.y);
                    
                    // Update selection rectangle
                    if let Some(rect) = &mut editor_state.selection_rect {
                        rect.2 = world_pos.x;
                        rect.3 = world_pos.y;
                    }
                    
                    // Handle gizmo dragging
                    if editor_state.gizmo_handle != crate::editor::state::GizmoHandle::None {
                        Self::handle_gizmo_drag(editor_state, world, undo_redo);
                    }
                }
            } else {
                // Mouse released
                if editor_state.is_dragging {
                    editor_state.is_dragging = false;
                    
                    // Finalize selection rectangle
                    if let Some((x1, y1, x2, y2)) = editor_state.selection_rect {
                        Self::select_entities_in_rect(editor_state, world, x1, y1, x2, y2);
                        editor_state.selection_rect = None;
                    }
                    
                    editor_state.gizmo_handle = crate::editor::state::GizmoHandle::None;
                }
            }
        }
    }
    
    fn screen_to_world(screen_pos: Pos2, editor_state: &EditorState) -> Pos2 {
        Pos2::new(
            (screen_pos.x - editor_state.viewport_pan.0) / editor_state.viewport_zoom,
            (screen_pos.y - editor_state.viewport_pan.1) / editor_state.viewport_zoom,
        )
    }
    
    fn handle_gizmo_drag(
        editor_state: &mut EditorState,
        world: &mut World,
        undo_redo: &mut crate::editor::undo_redo::UndoRedo,
    ) {
        use crate::ecs::components::{Position, Rotation, Scale};
        use specs::WorldExt;
        
        let delta_x = editor_state.drag_current_pos.0 - editor_state.drag_start_pos.0;
        let delta_y = editor_state.drag_current_pos.1 - editor_state.drag_start_pos.1;
        
        let mut positions = world.write_storage::<Position>();
        let mut rotations = world.write_storage::<Rotation>();
        let mut scales = world.write_storage::<Scale>();
        
        for entity in &editor_state.selected_entities {
            match editor_state.mode {
                EditorMode::Move => {
                    if let Some(pos) = positions.get_mut(*entity) {
                        let (snapped_x, snapped_y) = editor_state.snap_position(
                            pos.x + delta_x,
                            pos.y + delta_y,
                        );
                        pos.x = snapped_x;
                        pos.y = snapped_y;
                    }
                }
                EditorMode::Rotate => {
                    if let Some(rot) = rotations.get_mut(*entity) {
                        let angle = (delta_y / 100.0).atan2(delta_x / 100.0);
                        rot.angle += angle;
                    }
                }
                EditorMode::Scale => {
                    if let Some(scale) = scales.get_mut(*entity) {
                        scale.x += delta_x / 100.0;
                        scale.y += delta_y / 100.0;
                        scale.x = scale.x.max(0.1);
                        scale.y = scale.y.max(0.1);
                    }
                }
                _ => {}
            }
        }
        
        editor_state.drag_start_pos = editor_state.drag_current_pos;
    }
    
    fn select_entities_in_rect(
        editor_state: &mut EditorState,
        world: &World,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
    ) {
        use specs::{WorldExt, Join};
        use crate::ecs::components::Position;
        
        let min_x = x1.min(x2);
        let max_x = x1.max(x2);
        let min_y = y1.min(y2);
        let max_y = y1.max(y2);
        
        // Collect entities to select first to avoid borrow issues
        let mut entities_to_select = Vec::new();
        {
            let positions = world.read_storage::<Position>();
            let entities = world.entities();
            
            for (entity, pos) in (&entities, &positions).join() {
                if pos.x >= min_x && pos.x <= max_x && pos.y >= min_y && pos.y <= max_y {
                    entities_to_select.push(entity);
                }
            }
        }
        
        // Now select them
        let multi = false; // TODO: Check modifier keys
        for entity in entities_to_select {
            editor_state.select_entity(entity, multi);
        }
    }
}
