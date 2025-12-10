use egui::Ui;
use specs::{World, WorldExt, Join, Entity};
use crate::ecs::components::{Name, Position};
use crate::editor::state::EditorState;

pub struct EditorHierarchy;

impl EditorHierarchy {
    pub fn ui(ui: &mut Ui, editor_state: &mut EditorState, world: &mut World) {
        ui.heading("Hierarchy");
        
        // Search box with unique ID
        let mut search_text = String::new();
        ui.push_id("hierarchy_search", |ui| {
            ui.text_edit_singleline(&mut search_text);
        });
        
        ui.separator();
        
        // Scene root
        ui.collapsing("Scene", |ui| {
            let names = world.read_storage::<Name>();
            let entities = world.entities();
            
            let mut objects: Vec<(Entity, String)> = Vec::new();
            for (entity, name_comp) in (&entities, &names).join() {
                let name = name_comp.name.clone();
                if search_text.is_empty() || name.to_lowercase().contains(&search_text.to_lowercase()) {
                    objects.push((entity, name));
                }
            }
            objects.sort_by(|a, b| a.1.cmp(&b.1));
            
            for (entity, name) in objects {
                let is_selected = editor_state.selected_entities.contains(&entity);
                
                // Use entity ID to create unique widget IDs by pushing a unique ID context
                let entity_id = format!("entity_{:?}", entity);
                ui.push_id(entity_id, |ui| {
                    // Selectable label
                    if ui.selectable_label(is_selected, &name).clicked() {
                        // Multi-select with Ctrl/Shift - simplified for now
                        editor_state.select_entity(entity, false);
                    }
                });
            }
        });
        
        // Add object button
        ui.separator();
        if ui.button("+ Add Object").clicked() {
            Self::add_new_object(world, editor_state);
        }
    }
    
    fn add_new_object(world: &mut World, editor_state: &mut EditorState) {
        use specs::{Builder, Join};
        
        // Check existing names first
        let mut counter = 1;
        let mut name = format!("Object {}", counter);
        loop {
            let name_exists = {
                let names = world.read_storage::<Name>();
                let entities = world.entities();
                (&entities, &names).join().any(|(_, n)| n.name == name)
            };
            
            if !name_exists {
                break;
            }
            counter += 1;
            name = format!("Object {}", counter);
        }
        
        let entity = world.create_entity()
            .with(Name { name: name.clone() })
            .with(Position { x: 0.0, y: 0.0 })
            .with(crate::ecs::components::Rotation::default())
            .with(crate::ecs::components::Scale::default())
            .build();
        
        editor_state.select_entity(entity, false);
        editor_state.add_toast(format!("Created: {}", name), crate::editor::state::ToastLevel::Info);
    }
}
