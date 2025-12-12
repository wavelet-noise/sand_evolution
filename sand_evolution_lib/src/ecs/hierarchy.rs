use crate::ecs::components::{Children, Parent};
use specs::{Entity, World, WorldExt};

/// Detach `child` from its parent (if any), keeping the world consistent.
pub fn detach_from_parent(world: &mut World, child: Entity) {
    let maybe_parent = {
        let parents = world.read_storage::<Parent>();
        parents.get(child).map(|p| p.entity)
    };

    if let Some(parent) = maybe_parent {
        // Remove child from parent's children list.
        if let Some(children) = world.write_storage::<Children>().get_mut(parent) {
            children.entities.retain(|&e| e != child);
        }
        // Remove Parent component from child.
        let _ = world.write_storage::<Parent>().remove(child);
    }
}

/// Attach `child` under `parent`.
///
/// This will first detach `child` from its previous parent (if any),
/// then add it to `parent`'s `Children` list and set `child`'s `Parent`.
pub fn attach_child(world: &mut World, parent: Entity, child: Entity) {
    if parent == child {
        return;
    }

    detach_from_parent(world, child);

    // Ensure parent has a Children component.
    {
        let mut children_storage = world.write_storage::<Children>();
        if children_storage.get(parent).is_none() {
            let _ = children_storage.insert(parent, Children::default());
        }
        if let Some(children) = children_storage.get_mut(parent) {
            if !children.entities.contains(&child) {
                children.entities.push(child);
            }
        }
    }

    let _ = world
        .write_storage::<Parent>()
        .insert(child, Parent { entity: parent });
}

fn collect_subtree_dfs(world: &World, root: Entity, out: &mut Vec<Entity>) {
    out.push(root);

    let children_storage = world.read_storage::<Children>();
    if let Some(children) = children_storage.get(root) {
        for &ch in &children.entities {
            collect_subtree_dfs(world, ch, out);
        }
    }
}

/// Delete `root` and all its descendants (as defined by `Children`).
///
/// - Updates parent->children lists to remove deleted entities.
/// - Clears `Parent` links where relevant.
/// - Returns the list of deleted entities (including `root`).
pub fn delete_subtree(world: &mut World, root: Entity) -> Vec<Entity> {
    // Snapshot the subtree first (read-only pass).
    let subtree: Vec<Entity> = {
        let mut v = Vec::new();
        collect_subtree_dfs(world, root, &mut v);
        v
    };

    if subtree.is_empty() {
        return subtree;
    }

    // Detach all nodes from their parents so we don't leave dangling children pointers.
    // (This also removes Parent components on the nodes we detach.)
    for &e in &subtree {
        detach_from_parent(world, e);
    }

    // Delete children first, then parents.
    for &e in subtree.iter().rev() {
        world.delete_entity(e).ok();
    }

    subtree
}
