use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use std::cell::Cell;
use cgmath::{InnerSpace, Matrix, Matrix2, Matrix3, SquareMatrix, Vector2, Vector3};
use rhai::EvalAltResult;
use crate::CellTypeNotFound;
use crate::shared_state::SharedState;
use specs::{Builder, WorldExt, Join};

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
        rhai.register_fn("set_cell", move |v: Vector2::<f64>, t: i64| {
            moved_clone
                .borrow_mut()
                .set_pixel(v.x as i32, v.y as i32, t as u8);
        });
    }
    {
        rhai.register_fn("draw_line", move |v1: Vector2::<f64>, v2: Vector2::<f64>, t: i64| {
            draw_line(v1, v2, t as u8, shared_state_rc.clone());
        });
    }
    rhai.register_fn("type_id", move |name: &str| -> Result<i64, Box<rhai::EvalAltResult>> {
        if id_dict.contains_key(name) {
            Ok(id_dict[name] as i64)
        } else {
            Err(EvalAltResult::ErrorSystem(
                "SystemError".into(),
                Box::new(CellTypeNotFound{name: name.to_string()})
            ).into())
        }
    });
    rhai.register_fn("fract", move |v: f64| { v.fract() });
    rhai.register_fn("rand", move || -> i64 { crate::random::my_rand() });
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
                    (*state_ptr).set_temperature(x as crate::cs::PointType, y as crate::cs::PointType, temp as f32);
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
                    (*state_ptr).set_temperature(x as crate::cs::PointType, y as crate::cs::PointType, temp as f32);
                }
            }
        });
    });
    
    rhai.register_fn("set_temperature", |v: Vector2::<f64>, temp: f64| {
        STATE_PTR.with(|ptr| {
            let state_ptr = ptr.get();
            if !state_ptr.is_null() {
                unsafe {
                    (*state_ptr).set_temperature(v.x as crate::cs::PointType, v.y as crate::cs::PointType, temp as f32);
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
            use crate::ecs::components::{Name, Script, ScriptType};
            let mut world = world_clone.borrow_mut();
            world.create_entity()
                .with(Name { name: name.to_owned() })
                .with(Script {
                    script: "".to_owned(),
                    ast: None,
                    raw: true,
                    script_type: ScriptType::Entity,
                })
                .build();
            true
        });

        let world_clone = world_ref.clone();
        rhai.register_fn("set_object_script", move |name: &str, script: &str| -> bool {
            use specs::Join;
            use crate::ecs::components::{Name, Script};
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
                    return true;
                }
            }
            false
        });

        let world_clone = world_ref.clone();
        rhai.register_fn("get_object_script", move |name: &str| -> String {
            use specs::Join;
            use crate::ecs::components::{Name, Script};
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
            use specs::Join;
            use crate::ecs::components::Name;
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
                world.delete_entity(entity).ok();
                return true;
            }
            false
        });
    }

    rhai
        .register_type::<Vector2<f64>>()
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

    rhai
        .register_type::<Vector3<f64>>()
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

    rhai
        .register_type::<Matrix2<f64>>()
        .register_fn("mat2", Matrix2::<f64>::new)
        .register_fn("*", |m: Matrix2<f64>, v: Vector2<f64>| m * v)
        .register_fn("*", |a: Matrix2<f64>, b: Matrix2<f64>| a * b)
        .register_fn("transpose", |a: Matrix2<f64>| a.transpose())
        .register_fn("invert", |a: Matrix2<f64>| a.invert());

    rhai
        .register_type::<Matrix3<f64>>()
        .register_fn("mat3", Matrix3::<f64>::new)
        .register_fn("*", |m: Matrix3<f64>, v: Vector3<f64>| m * v)
        .register_fn("*", |a: Matrix3<f64>, b: Matrix3<f64>| a * b)
        .register_fn("transpose", |a: Matrix3<f64>| a.transpose())
        .register_fn("invert", |a: Matrix3<f64>| a.invert());
}

fn draw_line(start: Vector2<f64>, end: Vector2<f64>, t: u8, state: Rc<RefCell<SharedState>>)
{
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