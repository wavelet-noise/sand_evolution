struct InstanceUpdateSystem;
impl <'a> System<'a> for InstanceUpdateSystem {
    type SystemData = (
        ReadStorage<'a, Position>,
        ReadStorage<'a, Appearance>,
        WriteExpect<'a, Vec<Instance>>,
    );

    fn run(&mut self, (r_pos, r_appearance, mut instances): Self::SystemData) {
        instances.clear();
        instances.extend((&r_pos, &r_appearance).join().map(|(pos, appearance)| {
            Instance {
                offset: pos.0,
                origin: appearance.origin,
                scale: appearance.scale,
                rotation: appearance.rotation,
                color: appearance.color,
            }
        }));
    }
}
