use crate::ecs::components::{Name, Position, Rotation, Scale};
use crate::editor::state::EditorState;
use egui::Ui;
use specs::{Builder, World, WorldExt};

pub struct AddPanel;

impl AddPanel {
    pub fn ui(ui: &mut Ui, editor_state: &mut EditorState, world: &mut World) {
        ui.heading("Add Objects");

        ui.separator();

        ui.label("Primitives:");

        if ui.button("Empty Object").clicked() {
            Self::add_empty_object(world, editor_state);
        }

        if ui.button("Rectangle").clicked() {
            Self::add_rectangle(world, editor_state);
        }

        if ui.button("Circle").clicked() {
            Self::add_circle(world, editor_state);
        }

        ui.separator();

        ui.label("Drag objects here to add to scene");
    }

    fn add_empty_object(world: &mut World, editor_state: &mut EditorState) {
        let name = Self::generate_unique_name(world, "Object");
        let entity = world
            .create_entity()
            .with(Name { name: name.clone() })
            .with(Position { x: 0.0, y: 0.0 })
            .with(Rotation::default())
            .with(Scale::default())
            .build();

        editor_state.select_entity(entity, false);
        editor_state.add_toast(
            format!("Created: {}", name),
            crate::editor::state::ToastLevel::Info,
        );
    }

    fn add_rectangle(world: &mut World, editor_state: &mut EditorState) {
        let name = Self::generate_unique_name(world, "Rectangle");
        let entity = world
            .create_entity()
            .with(Name { name: name.clone() })
            .with(Position { x: 0.0, y: 0.0 })
            .with(Rotation::default())
            .with(Scale { x: 50.0, y: 50.0 })
            .build();

        editor_state.select_entity(entity, false);
        editor_state.add_toast(
            format!("Created: {}", name),
            crate::editor::state::ToastLevel::Info,
        );
    }

    fn add_circle(world: &mut World, editor_state: &mut EditorState) {
        let name = Self::generate_unique_name(world, "Circle");
        let entity = world
            .create_entity()
            .with(Name { name: name.clone() })
            .with(Position { x: 0.0, y: 0.0 })
            .with(Rotation::default())
            .with(Scale { x: 25.0, y: 25.0 })
            .build();

        editor_state.select_entity(entity, false);
        editor_state.add_toast(
            format!("Created: {}", name),
            crate::editor::state::ToastLevel::Info,
        );
    }

    fn generate_unique_name(world: &World, base: &str) -> String {
        use crate::ecs::components::Name;
        use specs::{Join, WorldExt};

        let names = world.read_storage::<Name>();
        let entities = world.entities();

        let mut counter = 1;
        loop {
            let name = if counter == 1 {
                base.to_string()
            } else {
                format!("{} {}", base, counter)
            };

            let exists = (&entities, &names).join().any(|(_, n)| n.name == name);

            if !exists {
                return name;
            }

            counter += 1;
        }
    }
}
