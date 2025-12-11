use egui::Ui;
use specs::{World, WorldExt, Join, Entity};
use crate::ecs::components::{Children, Name, Parent, Position};
use crate::editor::state::EditorState;
use std::collections::HashMap;

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
            // Build snapshots first (immutable borrows only), so we can freely mutate the world
            // later (e.g. when deleting entities).
            let (name_map, children_map, roots, any_matches) = {
                let names = world.read_storage::<Name>();
                let parents = world.read_storage::<Parent>();
                let children = world.read_storage::<Children>();
                let entities = world.entities();

                let mut name_map: HashMap<Entity, String> = HashMap::new();
                for (entity, name_comp) in (&entities, &names).join() {
                    name_map.insert(entity, name_comp.name.clone());
                }

                let mut children_map: HashMap<Entity, Vec<Entity>> = HashMap::new();
                for (entity, ch) in (&entities, &children).join() {
                    children_map.insert(entity, ch.entities.clone());
                }

                let search_lower = search_text.to_lowercase();

                // Roots = entities without a Parent component.
                let mut roots: Vec<Entity> = Vec::new();
                for (entity, name_comp) in (&entities, &names).join() {
                    if parents.get(entity).is_some() {
                        continue;
                    }
                    let name = name_comp.name.clone();
                    if search_text.is_empty() || name.to_lowercase().contains(&search_lower) {
                        roots.push(entity);
                    }
                }
                roots.sort_by(|a, b| {
                    let an = name_map.get(a).cloned().unwrap_or_default();
                    let bn = name_map.get(b).cloned().unwrap_or_default();
                    an.cmp(&bn)
                });

                // Search matches (even if entity has a parent).
                let mut any_matches: Vec<Entity> = Vec::new();
                if !search_text.is_empty() {
                    for (entity, name_comp) in (&entities, &names).join() {
                        let name = name_comp.name.clone();
                        if name.to_lowercase().contains(&search_lower) {
                            any_matches.push(entity);
                        }
                    }
                    any_matches.sort_by(|a, b| {
                        let an = name_map.get(a).cloned().unwrap_or_default();
                        let bn = name_map.get(b).cloned().unwrap_or_default();
                        an.cmp(&bn)
                    });
                }

                (name_map, children_map, roots, any_matches)
            };

            for entity in roots {
                Self::draw_entity_node(ui, editor_state, world, entity, &name_map, &children_map, &search_text);
            }

            if !search_text.is_empty() && !any_matches.is_empty() {
                ui.separator();
                ui.label("Search matches:");
                for entity in any_matches {
                    Self::draw_entity_row(ui, editor_state, world, entity, &name_map);
                }
            }
        });
        
        // Add object button
        ui.separator();
        if ui.button("+ Add Object").clicked() {
            Self::add_new_object(world, editor_state);
        }
    }

    fn draw_entity_node(
        ui: &mut Ui,
        editor_state: &mut EditorState,
        world: &mut World,
        entity: Entity,
        name_map: &HashMap<Entity, String>,
        children_map: &HashMap<Entity, Vec<Entity>>,
        search_text: &str,
    ) {
        let name = name_map
            .get(&entity)
            .cloned()
            .unwrap_or_else(|| "<unnamed>".to_owned());

        // If searching, only show full tree for nodes that match; otherwise show roots and their full subtrees.
        let search_lower = search_text.to_lowercase();
        let matches = search_text.is_empty() || name.to_lowercase().contains(&search_lower);

        let child_list = children_map.get(&entity).cloned().unwrap_or_default();

        if child_list.is_empty() {
            if matches {
                Self::draw_entity_row(ui, editor_state, world, entity, name_map);
            }
            return;
        }

        // Collapsible node for entities with children.
        let header_id = format!("node_{:?}", entity);
        ui.push_id(header_id, |ui| {
            egui::CollapsingHeader::new(name.clone())
                .default_open(true)
                .show(ui, |ui| {
                    for ch in child_list {
                        Self::draw_entity_node(ui, editor_state, world, ch, name_map, children_map, search_text);
                    }
                });

            // Also allow selecting/deleting the parent node via a row under the header.
            // This keeps interactions discoverable even when header is used for collapsing.
            if matches {
                ui.indent("node_actions", |ui| {
                    Self::draw_entity_row(ui, editor_state, world, entity, name_map);
                });
            }
        });
    }

    fn draw_entity_row(
        ui: &mut Ui,
        editor_state: &mut EditorState,
        world: &mut World,
        entity: Entity,
        name_map: &HashMap<Entity, String>,
    ) {
        let name = name_map
            .get(&entity)
            .cloned()
            .unwrap_or_else(|| "<unnamed>".to_owned());

        let is_selected = editor_state.selected_entities.contains(&entity);
        let entity_id = format!("entity_row_{:?}", entity);
        ui.push_id(entity_id, |ui| {
            ui.horizontal(|ui| {
                if ui.selectable_label(is_selected, &name).clicked() {
                    editor_state.select_entity(entity, false);
                }

                if ui.small_button("Delete").clicked() {
                    if name == "World Script" {
                        editor_state.add_toast(
                            "World Script cannot be deleted".to_owned(),
                            crate::editor::state::ToastLevel::Warning,
                        );
                        return;
                    }

                    let deleted = crate::ecs::hierarchy::delete_subtree(world, entity);
                    for e in deleted {
                        editor_state.selected_entities.remove(&e);
                    }
                    editor_state.add_toast(
                        format!("Deleted: {}", name),
                        crate::editor::state::ToastLevel::Info,
                    );
                }
            });
        });
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
