use specs::Entity;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    Select,
    Move,
    Rotate,
    Scale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoHandle {
    None,
    MoveX,
    MoveY,
    MoveXY,
    Rotate,
    ScaleX,
    ScaleY,
    ScaleXY,
}

pub struct EditorState {
    pub mode: EditorMode,
    pub selected_entities: HashSet<Entity>,
    pub gizmo_handle: GizmoHandle,
    pub is_dragging: bool,
    pub drag_start_pos: (f32, f32),
    pub drag_current_pos: (f32, f32),
    
    // Viewport state
    pub viewport_zoom: f32,
    pub viewport_pan: (f32, f32),
    pub show_grid: bool,
    pub show_toolbar: bool,
    pub snap_to_grid: bool,
    pub grid_size: f32,
    
    // Selection rectangle
    pub selection_rect: Option<(f32, f32, f32, f32)>, // (x1, y1, x2, y2)
    
    // Panel layout (calculated once at startup)
    pub hierarchy_pos: Option<(f32, f32)>,
    pub hierarchy_size: Option<(f32, f32)>,
    pub inspector_pos: Option<(f32, f32)>,
    pub inspector_size: Option<(f32, f32)>,
    
    // Toast notifications
    pub toasts: Vec<Toast>,
    
    // Scripts window request
    pub open_scripts_for_object: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub message: String,
    pub level: ToastLevel,
    pub lifetime: f32, // seconds remaining
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastLevel {
    Info,
    Warning,
    Error,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            mode: EditorMode::Select,
            selected_entities: HashSet::new(),
            gizmo_handle: GizmoHandle::None,
            is_dragging: false,
            drag_start_pos: (0.0, 0.0),
            drag_current_pos: (0.0, 0.0),
            viewport_zoom: 1.0,
            viewport_pan: (0.0, 0.0),
            show_grid: false,
            show_toolbar: false,
            snap_to_grid: true,
            grid_size: 10.0,
            selection_rect: None,
            hierarchy_pos: None,
            hierarchy_size: None,
            inspector_pos: None,
            inspector_size: None,
            toasts: Vec::new(),
            open_scripts_for_object: None,
        }
    }
    
    pub fn select_entity(&mut self, entity: Entity, multi: bool) {
        if multi {
            if self.selected_entities.contains(&entity) {
                self.selected_entities.remove(&entity);
            } else {
                self.selected_entities.insert(entity);
            }
        } else {
            self.selected_entities.clear();
            self.selected_entities.insert(entity);
        }
    }
    
    pub fn clear_selection(&mut self) {
        self.selected_entities.clear();
    }
    
    pub fn add_toast(&mut self, message: String, level: ToastLevel) {
        self.toasts.push(Toast {
            message,
            level,
            lifetime: 3.0, // 3 seconds default
        });
    }
    
    pub fn update_toasts(&mut self, delta_time: f32) {
        self.toasts.retain_mut(|toast| {
            toast.lifetime -= delta_time;
            toast.lifetime > 0.0
        });
    }
    
    pub fn snap_position(&self, x: f32, y: f32) -> (f32, f32) {
        if self.snap_to_grid {
            (
                (x / self.grid_size).round() * self.grid_size,
                (y / self.grid_size).round() * self.grid_size,
            )
        } else {
            (x, y)
        }
    }
}
