use crate::shared_state::SharedState;
use cgmath::{InnerSpace, Matrix, Matrix2, Matrix3, SquareMatrix, Vector2, Vector3};
use specs::{Builder, Join, WorldExt};
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

thread_local! {
    static STATE_PTR: Cell<*mut crate::State> = Cell::new(std::ptr::null_mut());
}

pub fn set_state_ptr(ptr: *mut crate::State) {
    STATE_PTR.with(|cell| {
        cell.set(ptr);
    });
}

pub fn register_rhai(
    rhai: &mut rhai::Engine,
    scope: &mut rhai::Scope,
    shared_state_rc: Rc<RefCell<SharedState>>,
    id_dict: HashMap<String, u8>,
    world_rc: Option<Rc<RefCell<specs::World>>>,
    script_log_rc: Rc<RefCell<VecDeque<String>>>,
    storage_rc: Option<Rc<RefCell<&mut crate::resources::rhai_resource::RhaiResourceStorage>>>,
) {
    //rhai_scope.push_constant("RES_X", dimensions.0);
    //rhai_scope.push_constant("RES_Y", dimensions.1);

    // String-based cell type overloads for set_cell (register first)
    // Helper function to convert string to ID, then call numeric set_cell
    {
        let moved_clone = shared_state_rc.clone();
        let id_dict_clone = id_dict.clone();
        rhai.register_fn("set_cell", move |x: i64, y: i64, t: &str| {
            if let Some(&cell_id) = id_dict_clone.get(t) {
                moved_clone
                    .borrow_mut()
                    .set_pixel(x as i32, y as i32, cell_id);
            }
            // Silently ignore if not found - this allows scripts to continue
        });
    }
    {
        let moved_clone = shared_state_rc.clone();
        let id_dict_clone = id_dict.clone();
        rhai.register_fn("set_cell", move |x: f64, y: f64, t: &str| {
            if let Some(&cell_id) = id_dict_clone.get(t) {
                moved_clone
                    .borrow_mut()
                    .set_pixel(x as i32, y as i32, cell_id);
            }
        });
    }
    {
        let moved_clone = shared_state_rc.clone();
        let id_dict_clone = id_dict.clone();
        rhai.register_fn("set_cell", move |v: Vector2<f64>, t: &str| {
            if let Some(&cell_id) = id_dict_clone.get(t) {
                moved_clone
                    .borrow_mut()
                    .set_pixel(v.x as i32, v.y as i32, cell_id);
            }
        });
    }
    // Numeric ID overloads for set_cell (register after string overloads to ensure priority)
    {
        let moved_clone = shared_state_rc.clone();
        rhai.register_fn("set_cell", move |x: i64, y: i64, t: i64| {
            moved_clone
                .borrow_mut()
                .set_pixel(x as i32, y as i32, t as u8);
        });
    }
    {
        let moved_clone = shared_state_rc.clone();
        rhai.register_fn("set_cell", move |x: f64, y: f64, t: i64| {
            moved_clone
                .borrow_mut()
                .set_pixel(x as i32, y as i32, t as u8);
        });
    }
    {
        let moved_clone = shared_state_rc.clone();
        rhai.register_fn("set_cell", move |v: Vector2<f64>, t: i64| {
            moved_clone
                .borrow_mut()
                .set_pixel(v.x as i32, v.y as i32, t as u8);
        });
    }
    // String-based cell type overload for draw_line (register first)
    {
        let id_dict_clone = id_dict.clone();
        let shared_state_clone = shared_state_rc.clone();
        rhai.register_fn(
            "draw_line",
            move |v1: Vector2<f64>, v2: Vector2<f64>, t: &str| {
                if let Some(&cell_id) = id_dict_clone.get(t) {
                    draw_line(v1, v2, cell_id, shared_state_clone.clone());
                }
            },
        );
    }
    // Numeric ID overload for draw_line (register after string overload to ensure priority)
    {
        let shared_state_clone = shared_state_rc.clone();
        rhai.register_fn(
            "draw_line",
            move |v1: Vector2<f64>, v2: Vector2<f64>, t: i64| {
                draw_line(v1, v2, t as u8, shared_state_clone.clone());
            },
        );
    }
    // Converter function: string cell type name -> numeric ID
    {
        let id_dict_clone = id_dict.clone();
        rhai.register_fn(
            "type_id",
            move |name: &str| -> i64 {
                id_dict_clone.get(name)
                    .copied()
                    .unwrap_or(0) as i64
            },
        );
    }
    // Alias for type_id - shorter name for convenience
    {
        let id_dict_clone = id_dict.clone();
        rhai.register_fn(
            "cell_id",
            move |name: &str| -> i64 {
                id_dict_clone.get(name)
                    .copied()
                    .unwrap_or(0) as i64
            },
        );
    }
    // Alternative function name for string-based set_cell to test if overloading is the issue
    {
        let moved_clone = shared_state_rc.clone();
        let id_dict_clone = id_dict.clone();
        rhai.register_fn("set_cell_str", move |x: i64, y: i64, t: &str| {
            if let Some(&cell_id) = id_dict_clone.get(t) {
                moved_clone
                    .borrow_mut()
                    .set_pixel(x as i32, y as i32, cell_id);
            }
        });
    }
    {
        let moved_clone = shared_state_rc.clone();
        let id_dict_clone = id_dict.clone();
        rhai.register_fn("set_cell_str", move |x: f64, y: f64, t: &str| {
            if let Some(&cell_id) = id_dict_clone.get(t) {
                moved_clone
                    .borrow_mut()
                    .set_pixel(x as i32, y as i32, cell_id);
            }
        });
    }
    {
        let moved_clone = shared_state_rc.clone();
        let id_dict_clone = id_dict.clone();
        rhai.register_fn("set_cell_str", move |v: Vector2<f64>, t: &str| {
            if let Some(&cell_id) = id_dict_clone.get(t) {
                moved_clone
                    .borrow_mut()
                    .set_pixel(v.x as i32, v.y as i32, cell_id);
            }
        });
    }

    rhai.register_fn("fract", move |v: f64| v.fract());
    rhai.register_fn("rand", move || -> i64 { crate::random::my_rand() });

    // Compatibility math functions for older scripts that use sin(x), cos(x), etc.
    // Newer scripts often use method forms like x.sin(), but many existing templates
    // rely on function-call style.
    rhai.register_fn("sin", |x: f64| x.sin());
    rhai.register_fn("sin", |x: i64| (x as f64).sin());
    rhai.register_fn("cos", |x: f64| x.cos());
    rhai.register_fn("cos", |x: i64| (x as f64).cos());
    rhai.register_fn("tan", |x: f64| x.tan());
    rhai.register_fn("tan", |x: i64| (x as f64).tan());
    rhai.register_fn("sqrt", |x: f64| x.sqrt());
    rhai.register_fn("abs", |x: f64| x.abs());
    rhai.register_fn("abs", |x: i64| x.abs());
    rhai.register_fn("min", |a: f64, b: f64| a.min(b));
    rhai.register_fn("min", |a: i64, b: i64| a.min(b));
    rhai.register_fn("max", |a: f64, b: f64| a.max(b));
    rhai.register_fn("max", |a: i64, b: i64| a.max(b));
    scope.push("time", 0f64);
    scope.push("GRID_WIDTH", 1024i64);
    scope.push("GRID_HEIGHT", 512i64);

    // Register set_temperature function - reads state pointer from thread_local
    // Overload for i64, i64, f64 (for integer loop variables)
    rhai.register_fn("set_temperature", |x: i64, y: i64, temp: f64| {
        STATE_PTR.with(|ptr| {
            let state_ptr = ptr.get();
            if !state_ptr.is_null() {
                unsafe {
                    (*state_ptr).set_temperature(
                        x as crate::cs::PointType,
                        y as crate::cs::PointType,
                        temp as f32,
                    );
                }
            }
        });
    });

    // Overload for f64, f64, f64 (for floating point coordinates)
    rhai.register_fn("set_temperature", |x: f64, y: f64, temp: f64| {
        STATE_PTR.with(|ptr| {
            let state_ptr = ptr.get();
            if !state_ptr.is_null() {
                unsafe {
                    (*state_ptr).set_temperature(
                        x as crate::cs::PointType,
                        y as crate::cs::PointType,
                        temp as f32,
                    );
                }
            }
        });
    });

    rhai.register_fn("set_temperature", |v: Vector2<f64>, temp: f64| {
        STATE_PTR.with(|ptr| {
            let state_ptr = ptr.get();
            if !state_ptr.is_null() {
                unsafe {
                    (*state_ptr).set_temperature(
                        v.x as crate::cs::PointType,
                        v.y as crate::cs::PointType,
                        temp as f32,
                    );
                }
            }
        });
    });

    // Register print function for script logging with circular buffer (max 30 entries)
    const MAX_LOG_ENTRIES: usize = 30;

    {
        let log_clone = script_log_rc.clone();
        rhai.register_fn("print", move |message: &str| {
            let mut log = log_clone.borrow_mut();
            log.push_back(message.to_owned());
            if log.len() > MAX_LOG_ENTRIES {
                log.pop_front();
            }
        });
    }

    // Register print for different types
    {
        let log_clone = script_log_rc.clone();
        rhai.register_fn("print", move |value: i64| {
            let mut log = log_clone.borrow_mut();
            log.push_back(value.to_string());
            if log.len() > MAX_LOG_ENTRIES {
                log.pop_front();
            }
        });
    }

    {
        let log_clone = script_log_rc.clone();
        rhai.register_fn("print", move |value: f64| {
            let mut log = log_clone.borrow_mut();
            log.push_back(value.to_string());
            if log.len() > MAX_LOG_ENTRIES {
                log.pop_front();
            }
        });
    }

    {
        let log_clone = script_log_rc.clone();
        rhai.register_fn("print", move |value: bool| {
            let mut log = log_clone.borrow_mut();
            log.push_back(value.to_string());
            if log.len() > MAX_LOG_ENTRIES {
                log.pop_front();
            }
        });
    }

    // Register print for Vector2
    {
        let log_clone = script_log_rc.clone();
        rhai.register_fn("print", move |v: Vector2<f64>| {
            let mut log = log_clone.borrow_mut();
            log.push_back(format!("vec2({}, {})", v.x, v.y));
            if log.len() > MAX_LOG_ENTRIES {
                log.pop_front();
            }
        });
    }

    // Register print for Vector3
    {
        let log_clone = script_log_rc.clone();
        rhai.register_fn("print", move |v: Vector3<f64>| {
            let mut log = log_clone.borrow_mut();
            log.push_back(format!("vec3({}, {}, {})", v.x, v.y, v.z));
            if log.len() > MAX_LOG_ENTRIES {
                log.pop_front();
            }
        });
    }

    // Functions for working with objects
    if let Some(world_ref) = world_rc {
        let world_clone = world_ref.clone();
        rhai.register_fn("create_object", move |name: &str| -> bool {
            use crate::ecs::components::{Name, Position, Rotation, Scale, Script, ScriptType};
            let mut world = world_clone.borrow_mut();
            world
                .create_entity()
                .with(Name {
                    name: name.to_owned(),
                })
                .with(Position { x: 0.0, y: 0.0 })
                .with(Rotation::default())
                .with(Scale::default())
                .with(Script {
                    script: "".to_owned(),
                    ast: None,
                    raw: true,
                    script_type: ScriptType::Entity,
                    run_once: false,
                    has_run: false,
                })
                .build();
            true
        });

        // Basic "entity access" helpers by name (transform + hierarchy)
        let world_clone = world_ref.clone();
        rhai.register_fn("entity_exists", move |name: &str| -> bool {
            use crate::ecs::components::Name;
            use specs::Join;
            let world = world_clone.borrow();
            let names = world.read_storage::<Name>();
            let entities = world.entities();
            (&entities, &names).join().any(|(_, n)| n.name == name)
        });

        let world_clone = world_ref.clone();
        rhai.register_fn("get_position", move |name: &str| -> Vector2<f64> {
            use crate::ecs::components::{Name, Position};
            use specs::Join;
            let world = world_clone.borrow();
            let names = world.read_storage::<Name>();
            let positions = world.read_storage::<Position>();
            let entities = world.entities();

            for (entity, n) in (&entities, &names).join() {
                if n.name == name {
                    if let Some(p) = positions.get(entity) {
                        return Vector2::new(p.x as f64, p.y as f64);
                    }
                    break;
                }
            }
            Vector2::new(0.0, 0.0)
        });

        let world_clone = world_ref.clone();
        rhai.register_fn("set_position", move |name: &str, x: f64, y: f64| -> bool {
            use crate::ecs::components::{Name, Position};
            use specs::Join;
            let mut world = world_clone.borrow_mut();
            let entities = world.entities();
            let names = world.read_storage::<Name>();
            let mut positions = world.write_storage::<Position>();

            for (entity, n) in (&entities, &names).join() {
                if n.name == name {
                    if let Some(p) = positions.get_mut(entity) {
                        p.x = x as f32;
                        p.y = y as f32;
                        return true;
                    }
                    // If missing Position, add it.
                    let _ = positions.insert(
                        entity,
                        Position {
                            x: x as f32,
                            y: y as f32,
                        },
                    );
                    return true;
                }
            }
            false
        });

        let world_clone = world_ref.clone();
        rhai.register_fn("set_position", move |name: &str, v: Vector2<f64>| -> bool {
            // Delegate to the scalar overload.
            let mut world = world_clone.borrow_mut();
            // local helper: find entity and set
            use crate::ecs::components::{Name, Position};
            use specs::Join;
            let entities = world.entities();
            let names = world.read_storage::<Name>();
            let mut positions = world.write_storage::<Position>();
            for (entity, n) in (&entities, &names).join() {
                if n.name == name {
                    if let Some(p) = positions.get_mut(entity) {
                        p.x = v.x as f32;
                        p.y = v.y as f32;
                        return true;
                    }
                    let _ = positions.insert(
                        entity,
                        Position {
                            x: v.x as f32,
                            y: v.y as f32,
                        },
                    );
                    return true;
                }
            }
            false
        });

        let world_clone = world_ref.clone();
        rhai.register_fn("get_rotation", move |name: &str| -> f64 {
            use crate::ecs::components::{Name, Rotation};
            use specs::Join;
            let world = world_clone.borrow();
            let names = world.read_storage::<Name>();
            let rotations = world.read_storage::<Rotation>();
            let entities = world.entities();
            for (entity, n) in (&entities, &names).join() {
                if n.name == name {
                    if let Some(r) = rotations.get(entity) {
                        return r.angle as f64;
                    }
                    break;
                }
            }
            0.0
        });

        let world_clone = world_ref.clone();
        rhai.register_fn("set_rotation", move |name: &str, angle: f64| -> bool {
            use crate::ecs::components::{Name, Rotation};
            use specs::Join;
            let mut world = world_clone.borrow_mut();
            let entities = world.entities();
            let names = world.read_storage::<Name>();
            let mut rotations = world.write_storage::<Rotation>();
            for (entity, n) in (&entities, &names).join() {
                if n.name == name {
                    if let Some(r) = rotations.get_mut(entity) {
                        r.angle = angle as f32;
                        return true;
                    }
                    let _ = rotations.insert(
                        entity,
                        Rotation {
                            angle: angle as f32,
                        },
                    );
                    return true;
                }
            }
            false
        });

        let world_clone = world_ref.clone();
        rhai.register_fn("get_scale", move |name: &str| -> Vector2<f64> {
            use crate::ecs::components::{Name, Scale};
            use specs::Join;
            let world = world_clone.borrow();
            let names = world.read_storage::<Name>();
            let scales = world.read_storage::<Scale>();
            let entities = world.entities();
            for (entity, n) in (&entities, &names).join() {
                if n.name == name {
                    if let Some(s) = scales.get(entity) {
                        return Vector2::new(s.x as f64, s.y as f64);
                    }
                    break;
                }
            }
            Vector2::new(1.0, 1.0)
        });

        let world_clone = world_ref.clone();
        rhai.register_fn("set_scale", move |name: &str, x: f64, y: f64| -> bool {
            use crate::ecs::components::{Name, Scale};
            use specs::Join;
            let mut world = world_clone.borrow_mut();
            let entities = world.entities();
            let names = world.read_storage::<Name>();
            let mut scales = world.write_storage::<Scale>();
            for (entity, n) in (&entities, &names).join() {
                if n.name == name {
                    if let Some(s) = scales.get_mut(entity) {
                        s.x = x as f32;
                        s.y = y as f32;
                        return true;
                    }
                    let _ = scales.insert(
                        entity,
                        Scale {
                            x: x as f32,
                            y: y as f32,
                        },
                    );
                    return true;
                }
            }
            false
        });

        let world_clone = world_ref.clone();
        rhai.register_fn("set_scale", move |name: &str, v: Vector2<f64>| -> bool {
            use crate::ecs::components::{Name, Scale};
            use specs::Join;
            let mut world = world_clone.borrow_mut();
            let entities = world.entities();
            let names = world.read_storage::<Name>();
            let mut scales = world.write_storage::<Scale>();
            for (entity, n) in (&entities, &names).join() {
                if n.name == name {
                    if let Some(s) = scales.get_mut(entity) {
                        s.x = v.x as f32;
                        s.y = v.y as f32;
                        return true;
                    }
                    let _ = scales.insert(
                        entity,
                        Scale {
                            x: v.x as f32,
                            y: v.y as f32,
                        },
                    );
                    return true;
                }
            }
            false
        });

        let world_clone = world_ref.clone();
        rhai.register_fn(
            "set_parent",
            move |child_name: &str, parent_name: &str| -> bool {
                use crate::ecs::components::Name;
                use specs::Join;

                if child_name == parent_name {
                    return false;
                }

                let mut world = world_clone.borrow_mut();
                let (parent, child) = {
                    let entities = world.entities();
                    let names = world.read_storage::<Name>();

                    let mut child = None;
                    let mut parent = None;
                    for (e, n) in (&entities, &names).join() {
                        if n.name == child_name {
                            child = Some(e);
                        } else if n.name == parent_name {
                            parent = Some(e);
                        }
                        if child.is_some() && parent.is_some() {
                            break;
                        }
                    }
                    (parent, child)
                };

                if let (Some(parent), Some(child)) = (parent, child) {
                    crate::ecs::hierarchy::attach_child(&mut world, parent, child);
                    true
                } else {
                    false
                }
            },
        );

        let world_clone = world_ref.clone();
        rhai.register_fn("unparent", move |child_name: &str| -> bool {
            use crate::ecs::components::Name;
            use specs::Join;

            let mut world = world_clone.borrow_mut();
            let child = {
                let entities = world.entities();
                let names = world.read_storage::<Name>();
                let mut child = None;
                for (e, n) in (&entities, &names).join() {
                    if n.name == child_name {
                        child = Some(e);
                        break;
                    }
                }
                child
            };

            if let Some(child) = child {
                crate::ecs::hierarchy::detach_from_parent(&mut world, child);
                true
            } else {
                false
            }
        });

        let world_clone = world_ref.clone();
        rhai.register_fn("children", move |parent_name: &str| -> rhai::Array {
            use crate::ecs::components::{Children, Name};
            use specs::Join;

            let world = world_clone.borrow();
            let entities = world.entities();
            let names = world.read_storage::<Name>();
            let children_storage = world.read_storage::<Children>();

            let mut parent = None;
            for (e, n) in (&entities, &names).join() {
                if n.name == parent_name {
                    parent = Some(e);
                    break;
                }
            }
            let Some(parent) = parent else {
                return rhai::Array::new();
            };

            let Some(children) = children_storage.get(parent) else {
                return rhai::Array::new();
            };

            let mut out = rhai::Array::new();
            for &ch in &children.entities {
                if let Some(n) = names.get(ch) {
                    out.push(rhai::Dynamic::from(n.name.clone()));
                }
            }
            out
        });

        let world_clone = world_ref.clone();
        rhai.register_fn(
            "set_object_script",
            move |name: &str, script: &str| -> bool {
                use crate::ecs::components::{Name, Script};
                use specs::Join;
                let world = world_clone.borrow_mut();
                let names = world.read_storage::<Name>();
                let entities = world.entities();

                // First, find the entity
                let mut target_entity = None;
                for (entity, name_comp) in (&entities, &names).join() {
                    if name_comp.name == name {
                        target_entity = Some(entity);
                        break;
                    }
                }

                // Then update the script
                if let Some(entity) = target_entity {
                    let mut scripts = world.write_storage::<Script>();
                    if let Some(script_comp) = scripts.get_mut(entity) {
                        script_comp.script = script.to_owned();
                        script_comp.raw = true;
                        script_comp.has_run = false;
                        return true;
                    }
                }
                false
            },
        );

        let world_clone = world_ref.clone();
        rhai.register_fn(
            "set_object_script_once",
            move |name: &str, script: &str| -> bool {
                use crate::ecs::components::{Name, Script};
                use specs::Join;
                let world = world_clone.borrow_mut();
                let names = world.read_storage::<Name>();
                let entities = world.entities();

                // First, find the entity
                let mut target_entity = None;
                for (entity, name_comp) in (&entities, &names).join() {
                    if name_comp.name == name {
                        target_entity = Some(entity);
                        break;
                    }
                }

                // Then update the script as a one-shot (runs on first tick after compile)
                if let Some(entity) = target_entity {
                    let mut scripts = world.write_storage::<Script>();
                    if let Some(script_comp) = scripts.get_mut(entity) {
                        script_comp.script = script.to_owned();
                        script_comp.raw = true;
                        script_comp.run_once = true;
                        script_comp.has_run = false;
                        return true;
                    }
                }
                false
            },
        );

        let world_clone = world_ref.clone();
        rhai.register_fn("get_object_script", move |name: &str| -> String {
            use crate::ecs::components::{Name, Script};
            use specs::Join;
            let world = world_clone.borrow();
            let names = world.read_storage::<Name>();
            let scripts = world.read_storage::<Script>();
            let entities = world.entities();

            for (entity, name_comp) in (&entities, &names).join() {
                if name_comp.name == name {
                    if let Some(script) = scripts.get(entity) {
                        return script.script.clone();
                    }
                }
            }
            "".to_owned()
        });

        let world_clone = world_ref.clone();
        rhai.register_fn("delete_object", move |name: &str| -> bool {
            use crate::ecs::components::Name;
            use specs::Join;
            if name == "World Script" {
                return false;
            }

            let mut world = world_clone.borrow_mut();
            let mut target_entity = None;
            {
                let names = world.read_storage::<Name>();
                let entities = world.entities();

                // First, find the entity
                for (entity, name_comp) in (&entities, &names).join() {
                    if name_comp.name == name {
                        target_entity = Some(entity);
                        break;
                    }
                }
            }

            // Then delete
            if let Some(entity) = target_entity {
                crate::ecs::hierarchy::delete_subtree(&mut world, entity);
                return true;
            }
            false
        });

        // Alias: prefer `delete_entity` naming in scripts.
        let world_clone = world_ref.clone();
        rhai.register_fn("delete_entity", move |name: &str| -> bool {
            if name == "World Script" {
                return false;
            }
            let mut world = world_clone.borrow_mut();
            use crate::ecs::components::Name;
            use specs::Join;

            let mut target_entity = None;
            {
                let names = world.read_storage::<Name>();
                let entities = world.entities();
                for (entity, name_comp) in (&entities, &names).join() {
                    if name_comp.name == name {
                        target_entity = Some(entity);
                        break;
                    }
                }
            }
            if let Some(entity) = target_entity {
                crate::ecs::hierarchy::delete_subtree(&mut world, entity);
                return true;
            }
            false
        });
    }

    rhai.register_type::<Vector2<f64>>()
        .register_fn("vec2", Vector2::<f64>::new)
        .register_set("x", |v: &mut Vector2<f64>, x: f64| v.x = x)
        .register_set("y", |v: &mut Vector2<f64>, y: f64| v.y = y)
        .register_get("x", |v: &mut Vector2<f64>| v.x)
        .register_get("y", |v: &mut Vector2<f64>| v.y)
        .register_fn("+", |a: Vector2<f64>, b: Vector2<f64>| a + b)
        .register_fn("-", |a: Vector2<f64>, b: Vector2<f64>| a - b)
        .register_fn("*", |v: Vector2<f64>, scalar: f64| v * scalar)
        .register_fn("magnitude", |v: &mut Vector2<f64>| v.magnitude())
        .register_fn("normalize", |v: &mut Vector2<f64>| v.normalize());

    rhai.register_type::<Vector3<f64>>()
        .register_fn("vec3", Vector3::<f64>::new)
        .register_set("x", |v: &mut Vector3<f64>, x: f64| v.x = x)
        .register_set("y", |v: &mut Vector3<f64>, y: f64| v.y = y)
        .register_set("z", |v: &mut Vector3<f64>, z: f64| v.z = z)
        .register_get("x", |v: &mut Vector3<f64>| v.x)
        .register_get("y", |v: &mut Vector3<f64>| v.y)
        .register_get("z", |v: &mut Vector3<f64>| v.z)
        .register_fn("+", |a: Vector3<f64>, b: Vector3<f64>| a + b)
        .register_fn("-", |a: Vector3<f64>, b: Vector3<f64>| a - b)
        .register_fn("*", |v: Vector3<f64>, scalar: f64| v * scalar)
        .register_fn("magnitude", |v: &mut Vector3<f64>| v.magnitude())
        .register_fn("normalize", |v: &mut Vector3<f64>| v.normalize())
        .register_fn("dot", |a: Vector3<f64>, b: Vector3<f64>| a.dot(b))
        .register_fn("cross", |a: Vector3<f64>, b: Vector3<f64>| a.cross(b));

    rhai.register_type::<Matrix2<f64>>()
        .register_fn("mat2", Matrix2::<f64>::new)
        .register_fn("*", |m: Matrix2<f64>, v: Vector2<f64>| m * v)
        .register_fn("*", |a: Matrix2<f64>, b: Matrix2<f64>| a * b)
        .register_fn("transpose", |a: Matrix2<f64>| a.transpose())
        .register_fn("invert", |a: Matrix2<f64>| a.invert());

    rhai.register_type::<Matrix3<f64>>()
        .register_fn("mat3", Matrix3::<f64>::new)
        .register_fn("*", |m: Matrix3<f64>, v: Vector3<f64>| m * v)
        .register_fn("*", |a: Matrix3<f64>, b: Matrix3<f64>| a * b)
        .register_fn("transpose", |a: Matrix3<f64>| a.transpose())
        .register_fn("invert", |a: Matrix3<f64>| a.invert());
}

fn draw_line(start: Vector2<f64>, end: Vector2<f64>, t: u8, state: Rc<RefCell<SharedState>>) {
    let mut x0 = start.x as i64;
    let mut y0 = start.y as i64;
    let x1 = end.x as i64;
    let y1 = end.y as i64;

    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();

    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };

    let mut err = dx - dy;

    loop {
        state.borrow_mut().set_pixel(x0 as i32, y0 as i32, t);

        if x0 == x1 && y0 == y1 {
            break;
        }

        let e2 = 2 * err;

        if e2 > -dy {
            err -= dy;
            x0 += sx;
        }
        if e2 < dx {
            err += dx;
            y0 += sy;
        }
    }
}
