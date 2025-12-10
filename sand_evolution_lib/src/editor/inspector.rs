use egui::{Ui, DragValue};
use specs::{World, WorldExt, ReadStorage, WriteStorage, Entity};
use crate::ecs::components::{Position, Rotation, Scale, Name, Script};
use crate::editor::state::EditorState;

pub struct EditorInspector;

impl EditorInspector {
    pub fn ui(ui: &mut Ui, editor_state: &mut EditorState, world: &mut World) {
        ui.heading("Inspector");
        
        if editor_state.selected_entities.is_empty() {
            ui.label("No selection");
            return;
        }
        
        let selected: Vec<Entity> = editor_state.selected_entities.iter().copied().collect();
        
        if selected.len() == 1 {
            Self::show_single_entity(ui, selected[0], world, editor_state);
        } else {
            Self::show_multiple_entities(ui, &selected, world);
        }
    }
    
    fn show_single_entity(ui: &mut Ui, entity: Entity, world: &mut World, editor_state: &mut EditorState) {
        // Show name
        {
            let mut names = world.write_storage::<Name>();
            if let Some(name) = names.get_mut(entity) {
                ui.label("Name:");
                ui.text_edit_singleline(&mut name.name);
            }
        }
        
        ui.separator();
        
        // Transform section
        ui.collapsing("Transform", |ui| {
            let mut positions = world.write_storage::<Position>();
            let mut rotations = world.write_storage::<Rotation>();
            let mut scales = world.write_storage::<Scale>();
            
            // Position
            if let Some(pos) = positions.get_mut(entity) {
                ui.horizontal(|ui| {
                    ui.label("Position:");
                    ui.add(DragValue::new(&mut pos.x).speed(1.0).prefix("X: "));
                    ui.add(DragValue::new(&mut pos.y).speed(1.0).prefix("Y: "));
                });
            } else {
                ui.label("No Position component");
            }
            
            // Rotation
            if let Some(rot) = rotations.get_mut(entity) {
                ui.horizontal(|ui| {
                    ui.label("Rotation:");
                    ui.add(DragValue::new(&mut rot.angle)
                        .speed(0.01)
                        .prefix("Angle: ")
                        .suffix(" rad"));
                    let degrees = rot.angle.to_degrees();
                    ui.label(format!("({:.1}°)", degrees));
                });
            } else {
                ui.label("No Rotation component");
            }
            
            // Scale
            if let Some(scale) = scales.get_mut(entity) {
                ui.horizontal(|ui| {
                    ui.label("Scale:");
                    ui.add(DragValue::new(&mut scale.x).speed(0.1).prefix("X: "));
                    ui.add(DragValue::new(&mut scale.y).speed(0.1).prefix("Y: "));
                });
            } else {
                ui.label("No Scale component");
            }
        });
        
        ui.separator();
        
        // Script section
        // Get object name before collapsing
        let object_name = {
            let names = world.read_storage::<Name>();
            names.get(entity).map(|n| n.name.clone())
        };
        
        use std::cell::RefCell;
        let script_editor_object_name = RefCell::new(None::<String>);
        
        ui.collapsing("Script", |ui| {
            let scripts = world.read_storage::<Script>();
            
            if let Some(script) = scripts.get(entity) {
                // Script info
                ui.horizontal(|ui| {
                    ui.label("Status:");
                    if script.ast.is_some() {
                        ui.label("Compiled");
                    } else {
                        ui.label("Not compiled");
                    }
                });
                
                ui.horizontal(|ui| {
                    ui.label("Raw mode:");
                    ui.label(format!("{}", script.raw));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Code length:");
                    ui.label(format!("{} chars", script.script.len()));
                });
                
                ui.separator();
                
                // Button to open script editor
                if let Some(ref name) = object_name {
                    if ui.button("Open Script Editor").clicked() {
                        *script_editor_object_name.borrow_mut() = Some(name.clone());
                    }
                }
            } else {
                ui.label("No Script component");
                let mut scripts = world.write_storage::<Script>();
                if ui.button("Add Script Component").clicked() {
                    use crate::ecs::components::ScriptType;
                    scripts.insert(entity, Script {
                        script: "".to_owned(),
                        ast: None,
                        raw: true,
                        script_type: ScriptType::Entity,
                    });
                }
            }
        });
        
        // Handle script editor opening after collapsing
        let name_to_open = script_editor_object_name.borrow_mut().take();
        if let Some(name) = name_to_open {
            editor_state.open_scripts_for_object = Some(name);
        }
        
        ui.separator();
        
        // Visual section
        ui.collapsing("Visual", |ui| {
            // TODO: Add color, sprite, layer components when available
            ui.label("Visual properties - coming soon");
        });
    }
    
    fn show_multiple_entities(ui: &mut Ui, entities: &[Entity], world: &mut World) {
        ui.label(format!("{} objects selected", entities.len()));
        
        ui.separator();
        
        // Multi-edit transform
        ui.collapsing("Transform (Multi-edit)", |ui| {
            let positions = world.read_storage::<Position>();
            let mut rotations = world.write_storage::<Rotation>();
            let mut scales = world.write_storage::<Scale>();
            
            // Check if all have position
            let all_have_position = entities.iter()
                .all(|e| positions.get(*e).is_some());
            
            if all_have_position {
                // Calculate average position
                let mut avg_x = 0.0;
                let mut avg_y = 0.0;
                let mut count = 0;
                
                for entity in entities {
                    if let Some(pos) = positions.get(*entity) {
                        avg_x += pos.x;
                        avg_y += pos.y;
                        count += 1;
                    }
                }
                
                if count > 0 {
                    avg_x /= count as f32;
                    avg_y /= count as f32;
                    
                    ui.horizontal(|ui| {
                        ui.label("Average Position:");
                        ui.label(format!("X: {:.1}, Y: {:.1}", avg_x, avg_y));
                    });
                }
            }
            
            // Rotation - show mixed if different
            let rotations_vec: Vec<_> = entities.iter()
                .filter_map(|e| rotations.get(*e))
                .collect();
            
            if !rotations_vec.is_empty() {
                let first_angle = rotations_vec[0].angle;
                let all_same = rotations_vec.iter().all(|r| r.angle == first_angle);
                
                if all_same {
                    ui.horizontal(|ui| {
                        ui.label("Rotation:");
                        ui.label(format!("{:.2} rad ({:.1}°)", first_angle, first_angle.to_degrees()));
                    });
                } else {
                    ui.label("Rotation: (Mixed values)");
                }
            }
            
            // Scale - show mixed if different
            let scales_vec: Vec<_> = entities.iter()
                .filter_map(|e| scales.get(*e))
                .collect();
            
            if !scales_vec.is_empty() {
                let first_scale = scales_vec[0];
                let all_same = scales_vec.iter().all(|s| s.x == first_scale.x && s.y == first_scale.y);
                
                if all_same {
                    ui.horizontal(|ui| {
                        ui.label("Scale:");
                        ui.label(format!("X: {:.2}, Y: {:.2}", first_scale.x, first_scale.y));
                    });
                } else {
                    ui.label("Scale: (Mixed values)");
                }
            }
        });
    }
}
