// Physics integration for combat - connects physics collisions to damage events
use bevy::prelude::*;
use crate::components::*;
use crate::damage::DamageEvent;
use crate::states::Health;

// Component to mark entities that deal collision damage
#[derive(Component)]
pub struct CollisionDamage {
    pub damage: f32,
    pub damage_type: DamageType,
    pub knockback_force: f32,
    pub can_damage_allies: bool,
}

// Component for projectile physics
#[derive(Component)]
pub struct ProjectilePhysics {
    pub velocity: Vec3,
    pub gravity_scale: f32,
    pub drag: f32,
    pub homing_strength: f32,
    pub target: Option<Entity>,
}

// Component for melee attack hitboxes
#[derive(Component)]
pub struct MeleeHitbox {
    pub damage: f32,
    pub damage_type: DamageType,
    pub active: bool,
    pub lifetime: f32,
    pub hit_entities: Vec<Entity>,
    pub owner: Entity,
}

// Component for area of effect damage zones
#[derive(Component)]
pub struct AreaOfEffectDamage {
    pub center: Vec3,
    pub radius: f32,
    pub damage: f32,
    pub damage_type: DamageType,
    pub duration: f32,
    pub tick_rate: f32,
    pub time_since_tick: f32,
    pub owner: Entity,
    pub affects_allies: bool,
}

// Plugin for physics-combat integration
pub struct CombatPhysicsPlugin;

impl Plugin for CombatPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                update_projectile_physics,
                handle_projectile_collisions,
                handle_melee_collisions,
                handle_area_damage,
                apply_knockback,
                cleanup_expired_hitboxes,
            ));
    }
}

/// Update projectile physics
pub fn update_projectile_physics(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut ProjectilePhysics, &Projectile)>,
    target_query: Query<&Transform, Without<Projectile>>,
) {
    for (mut transform, mut physics, _projectile) in query.iter_mut() {
        // Apply velocity
        transform.translation += physics.velocity * time.delta_secs();
        
        // Apply gravity
        physics.velocity.y -= 9.8 * physics.gravity_scale * time.delta_secs();
        
        // Apply drag
        let drag = physics.drag;
        physics.velocity *= 1.0 - (drag * time.delta_secs());
        
        // Apply homing if target exists
        if let Some(target_entity) = physics.target {
            if let Ok(target_transform) = target_query.get(target_entity) {
                let direction = (target_transform.translation - transform.translation).normalize();
                physics.velocity = physics.velocity.lerp(
                    direction * physics.velocity.length(),
                    physics.homing_strength * time.delta_secs(),
                );
            }
        }
        
        // Point projectile in direction of travel
        if physics.velocity.length() > 0.1 {
            transform.look_to(physics.velocity.normalize(), Vec3::Y);
        }
    }
}

/// Handle projectile collisions
pub fn handle_projectile_collisions(
    mut commands: Commands,
    mut damage_events: EventWriter<DamageEvent>,
    projectile_query: Query<(Entity, &Transform, &Projectile, Option<&CollisionDamage>)>,
    target_query: Query<(Entity, &Transform, &Team, Option<&Shield>), Without<Projectile>>,
) {
    for (proj_entity, proj_transform, projectile, collision_damage) in projectile_query.iter() {
        for (target_entity, target_transform, _target_team, _shield) in target_query.iter() {
            // Skip friendly fire unless enabled
            if let Some(damage) = collision_damage {
                if !damage.can_damage_allies && target_entity == projectile.owner {
                    continue;
                }
            }
            
            // Check collision distance
            let distance = proj_transform.translation.distance(target_transform.translation);
            if distance < 1.0 {  // Simple sphere collision, radius = 1.0
                // Calculate damage
                let damage_amount = collision_damage
                    .map(|cd| cd.damage)
                    .unwrap_or(projectile.damage);
                
                let damage_type = collision_damage
                    .map(|cd| cd.damage_type.clone())
                    .unwrap_or(projectile.damage_type.clone());
                
                // Send damage event
                damage_events.write(DamageEvent {
                    attacker: projectile.owner,
                    target: target_entity,
                    amount: damage_amount,
                    damage_type,
                    is_critical: false,  // Could calculate crit chance here
                });
                
                // Handle area damage if applicable
                if let Some(area) = &projectile.area_damage {
                    spawn_area_damage(&mut commands, proj_transform.translation, area, projectile.owner);
                }
                
                // Destroy projectile on hit (unless it pierces)
                if projectile.pierce_count == 0 {
                    commands.entity(proj_entity).despawn();
                }
                
                break;  // Hit one target per frame
            }
        }
    }
}

/// Handle melee attack collisions
pub fn handle_melee_collisions(
    time: Res<Time>,
    mut damage_events: EventWriter<DamageEvent>,
    mut melee_query: Query<(&Transform, &mut MeleeHitbox)>,
    target_query: Query<(Entity, &Transform, &Team), Without<MeleeHitbox>>,
) {
    for (melee_transform, mut hitbox) in melee_query.iter_mut() {
        if !hitbox.active {
            continue;
        }
        
        hitbox.lifetime -= time.delta_secs();
        if hitbox.lifetime <= 0.0 {
            hitbox.active = false;
            hitbox.hit_entities.clear();
            continue;
        }
        
        for (target_entity, target_transform, _target_team) in target_query.iter() {
            // Skip if already hit this entity
            if hitbox.hit_entities.contains(&target_entity) {
                continue;
            }
            
            // Skip self
            if target_entity == hitbox.owner {
                continue;
            }
            
            // Check collision distance (melee range)
            let distance = melee_transform.translation.distance(target_transform.translation);
            if distance < 2.0 {  // Melee range
                // Send damage event
                damage_events.write(DamageEvent {
                    attacker: hitbox.owner,
                    target: target_entity,
                    amount: hitbox.damage,
                    damage_type: hitbox.damage_type.clone(),
                    is_critical: false,
                });
                
                // Remember we hit this entity
                hitbox.hit_entities.push(target_entity);
            }
        }
    }
}

/// Handle area of effect damage
pub fn handle_area_damage(
    time: Res<Time>,
    mut commands: Commands,
    mut damage_events: EventWriter<DamageEvent>,
    mut aoe_query: Query<(Entity, &mut AreaOfEffectDamage)>,
    target_query: Query<(Entity, &Transform, &Team)>,
) {
    for (aoe_entity, mut aoe) in aoe_query.iter_mut() {
        aoe.duration -= time.delta_secs();
        
        if aoe.duration <= 0.0 {
            commands.entity(aoe_entity).despawn();
            continue;
        }
        
        aoe.time_since_tick += time.delta_secs();
        
        if aoe.time_since_tick >= aoe.tick_rate {
            aoe.time_since_tick = 0.0;
            
            // Find all entities in range
            for (target_entity, target_transform, _target_team) in target_query.iter() {
                // Skip owner unless friendly fire is enabled
                if !aoe.affects_allies && target_entity == aoe.owner {
                    continue;
                }
                
                let distance = aoe.center.distance(target_transform.translation);
                if distance <= aoe.radius {
                    // Calculate damage falloff
                    let falloff = 1.0 - (distance / aoe.radius) * 0.5;  // 50% damage at edge
                    let damage = aoe.damage * falloff;
                    
                    damage_events.write(DamageEvent {
                        attacker: aoe.owner,
                        target: target_entity,
                        amount: damage,
                        damage_type: aoe.damage_type.clone(),
                        is_critical: false,
                    });
                }
            }
        }
    }
}

/// Apply knockback from attacks
pub fn apply_knockback(
    mut damage_events: EventReader<DamageEvent>,
    collision_damage_query: Query<&CollisionDamage>,
    mut q: ParamSet<(
        Query<&Transform, With<CollisionDamage>>,   // attackers
        Query<&mut Transform, Without<CollisionDamage>>, // targets
    )>,
) {
    for event in damage_events.read() {
        // Guard against self-damage edge case
        if event.attacker == event.target {
            continue;
        }
        
        if let Ok(collision_damage) = collision_damage_query.get(event.attacker) {
            // Get attacker transform using the first query in ParamSet and store the position
            let attacker_position = if let Ok(attacker_transform) = q.p0().get(event.attacker) {
                attacker_transform.translation
            } else {
                continue;
            };
            
            // Now get target transform using the second query in ParamSet  
            if let Ok(mut target_transform) = q.p1().get_mut(event.target) {
                // Calculate knockback direction
                let direction = (target_transform.translation - attacker_position).normalize();
                let knockback = direction * collision_damage.knockback_force;
                
                // Apply knockback (simplified - would need velocity component in real implementation)
                target_transform.translation += knockback * 0.1;
            }
        }
    }
}

/// Cleanup expired melee hitboxes
pub fn cleanup_expired_hitboxes(
    mut commands: Commands,
    query: Query<(Entity, &MeleeHitbox)>,
) {
    for (entity, hitbox) in query.iter() {
        if !hitbox.active && hitbox.lifetime <= 0.0 {
            commands.entity(entity).remove::<MeleeHitbox>();
        }
    }
}

// Helper functions

/// Spawn an area damage zone
pub fn spawn_area_damage(
    commands: &mut Commands,
    position: Vec3,
    area: &AreaDamage,
    owner: Entity,
) {
    commands.spawn((
        Name::new("AreaDamage"),
        AreaOfEffectDamage {
            center: position,
            radius: area.radius,
            damage: area.radius * (1.0 - area.falloff),  // Base damage
            damage_type: DamageType::Magic,  // Default to magic for AOE
            duration: 3.0,  // 3 second duration
            tick_rate: 0.5,  // Damage every 0.5 seconds
            time_since_tick: 0.0,
            owner,
            affects_allies: area.friendly_fire,
        },
        Transform::from_translation(position),
    ));
}

/// Create a melee attack hitbox
pub fn create_melee_hitbox(
    commands: &mut Commands,
    owner: Entity,
    damage: f32,
    damage_type: DamageType,
) -> Entity {
    commands.spawn((
        Name::new("MeleeHitbox"),
        MeleeHitbox {
            damage,
            damage_type,
            active: true,
            lifetime: 0.2,  // Active for 0.2 seconds
            hit_entities: Vec::new(),
            owner,
        },
        Transform::default(),
    )).id()
}

/// Create a projectile with physics
pub fn create_physics_projectile(
    commands: &mut Commands,
    owner: Entity,
    position: Vec3,
    direction: Vec3,
    speed: f32,
    damage: f32,
    damage_type: DamageType,
) -> Entity {
    commands.spawn((
        Name::new("Projectile"),
        Projectile {
            owner,
            damage,
            damage_type,
            speed,
            lifetime: 5.0,
            remaining_lifetime: 5.0,
            pierce_count: 0,
            area_damage: None,
        },
        ProjectilePhysics {
            velocity: direction.normalize() * speed,
            gravity_scale: 0.2,  // Slight arc
            drag: 0.1,
            homing_strength: 0.0,
            target: None,
        },
        Transform::from_translation(position),
    )).id()
}