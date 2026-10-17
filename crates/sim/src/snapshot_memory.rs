//! Retained snapshot payload accounting, not allocator/footprint measurement.
//! Vec/String capacities and inline sizes are counted once. BTreeMap has no
//! capacity API: budget 1024 bytes per entry (conservative node allowance for
//! String/f32 on supported 64-bit targets), plus key capacities. Allocator
//! rounding/metadata and temporary clones are not included. No serialization.
//! Exhaustive destructuring forces review when a snapshot struct gains fields;
//! Copy bounds ensure fields classified inline cannot own heap allocations.
use crate::*;
use std::mem::size_of;

fn inline<T: Copy>(_: &T) {}
fn plain_vec<T: Copy>(v: &Vec<T>) -> usize {
    v.capacity().saturating_mul(size_of::<T>())
}
fn nested_vec<T>(v: &Vec<T>, heap: impl Fn(&T) -> usize) -> usize {
    v.iter()
        .fold(v.capacity().saturating_mul(size_of::<T>()), |n, x| {
            n.saturating_add(heap(x))
        })
}

macro_rules! account {
    ($fn:ident, $ty:path, { $($owned:ident => $bytes:expr,)* }; $($plain:ident),* $(,)?) => {
        fn $fn(value: &$ty) -> usize {
            let $ty { $($owned,)* $($plain,)* } = value;
            $(inline($plain);)*
            0usize $(.saturating_add($bytes))*
        }
    };
}

impl Snapshot {
    /// Estimated owned payload including this struct, not exact allocator bytes.
    pub fn owned_bytes(&self) -> usize {
        size_of::<Self>().saturating_add(snapshot(self))
    }
}

fn audience(value: &EventAudience) -> usize {
    match value {
        EventAudience::All | EventAudience::AllExcept(_) | EventAudience::Client(_) => 0,
        EventAudience::Clients(ids) => plain_vec(ids),
    }
}
fn menu_kind(value: &MenuCommandKind) -> usize {
    match value {
        MenuCommandKind::Open(name) => name.capacity(),
        MenuCommandKind::ClosePopup | MenuCommandKind::CloseInGame => 0,
    }
}

account!(snapshot, Snapshot, {
    players => plain_vec(players),
    projectiles => plain_vec(projectiles),
    meta => snapshot_meta(meta),
}; tick);

account!(snapshot_meta, SnapshotMeta, {
    objectives => self::objectives(objectives),
    clients => nested_vec(clients, |(_, value)| client(value)),
    journal => nested_vec(journal, event),
    entity_events => nested_vec(entity_events, entity_event),
    pellet_fx => plain_vec(pellet_fx),
    sound_aliases => nested_vec(sound_aliases, |(_, value)| value.capacity()),
    effect_names => nested_vec(effect_names, |(_, value)| value.capacity()),
    hud_materials => nested_vec(hud_materials, |(_, value)| value.capacity()),
    hud_strings => nested_vec(hud_strings, |(_, value)| value.capacity()),
    world_objects => self::world_objects(world_objects),
    area_entities => area_entities.as_ref().map_or(0, area),
    entity_dobjs => nested_vec(entity_dobjs, |(_, value)| dobj(value)),
    entities => plain_vec(entities),
    script_movers => plain_vec(script_movers),
    entity_kernel => kernel(entity_kernel),
    item_ammo => plain_vec(item_ammo),
    item_pickups => plain_vec(item_pickups),
}; phase, match_elapsed_ms, prematch, score_limit, time_limit_ms, kind, rng, corpses);

account!(client, ClientSnapshotMeta, {
    shield_collision => shield_collision.as_ref().map_or(0, self::shield_collision),
    loadout => loadout.as_ref().map_or(0, self::loadout),
    ammo_by_weapon => plain_vec(ammo_by_weapon),
    taped_mag_spent => plain_vec(taped_mag_spent),
    hud_archival => plain_vec(hud_archival),
    hud_current => plain_vec(hud_current),
    client_dvars => nested_vec(client_dvars, |(a, b)| a.capacity().saturating_add(b.capacity())),
    shellshock => shellshock.as_ref().map_or(0, shock),
    view_effects => view(view_effects),
    menu_commands => nested_vec(menu_commands, menu),
    location_selection => location_selection.as_ref().map_or(0, location),
}; shield, controls, weapon_lock, killcam_hud, lifecycle, god_mode, life_sequence, item_use_spawn_ms, item_use_entity, ammo_clip, ammo_stock, score, kills, deaths, kill_streak, radar, radar_blocked, remote_missile, linked_weapon_view, weapon_shot_count, burst_latch, burst_latch_secondary, rechamber_pending, rechamber_pending_secondary, pending_brass, dead_since_tick, look_at_killer_yaw, name, ffa_team, client_state_team, rank, prestige, player_card_icon, player_card_title, player_card_nameplate);

account!(shield_collision, ShieldCarrierCollision, {
    bones => plain_vec(bones),
}; normal);

account!(loadout, LoadoutSpec, {
}; class_id, revision, primary, secondary, primary_attachments, secondary_attachments, lethal, tactical, perks);

account!(objectives, ObjectiveMatch, {
    compass => nested_vec(compass, self::compass),
    vehicles => nested_vec(vehicles, vehicle),
    vehicle_targets => plain_vec(vehicle_targets),
    server_info => nested_vec(server_info, |(a, b)| a.capacity().saturating_add(b.capacity())),
    ambient => ambient.as_ref().map_or(0, self::ambient),
    ac130_ambient => ac130_ambient.as_ref().map_or(0, self::ambient),
    rumble_aliases => nested_vec(rumble_aliases, |(_, value)| value.capacity()),
    effects => nested_vec(effects, effect),
    earthquakes => plain_vec(earthquakes),
    naked_vision => naked_vision.as_ref().map_or(0, vision),
    thermal_vision => thermal_vision.as_ref().map_or(0, vision),
    thermal_body_material => thermal_body_material.capacity(),
    missile_vision => missile_vision.as_ref().map_or(0, vision),
    night_vision => night_vision.as_ref().map_or(0, vision),
    pain_vision => pain_vision.as_ref().map_or(0, vision),
}; scores, game_end_time, slow_motion, scripted_effects, fog);

account!(effect, ScriptEffect, {
}; id, effect, origin, forward, up, start_ms, repeat_ms, cull_distance, viewers);

account!(compass, CompassObjective, {
    icon => icon.capacity(),
}; index, state, origin, team, viewer);

account!(vehicle, CompassVehicle, {
    icons => icons.iter().map(String::capacity).fold(0, usize::saturating_add),
}; origin, yaw, owner, team, size);

account!(ambient, ScriptAmbient, {
    alias => alias.as_ref().map_or(0, String::capacity),
}; start_ms, end_ms);

account!(vision, VisionChange, {
    name => name.capacity(),
}; duration_ms, set_ms);

account!(view, ViewEffects, {
    naked_vision => naked_vision.as_ref().map_or(0, vision),
    thermal_vision => thermal_vision.as_ref().map_or(0, vision),
    missile_vision => missile_vision.as_ref().map_or(0, vision),
    night_vision => night_vision.as_ref().map_or(0, vision),
    pain_vision => pain_vision.as_ref().map_or(0, vision),
}; depth_of_field, blur);

account!(location, LocationSelection, {
    material => material.capacity(),
}; choose_direction, radius);

account!(menu, MenuCommand, {
    kind => menu_kind(kind),
}; serial);

account!(world_objects, WorldObjectSnapshot, {
    glass_pieces => plain_vec(glass_pieces),
    destructible_loop_sounds => plain_vec(destructible_loop_sounds),
}; as_of_ms, map_round_epoch, fracture_profile_version);

account!(area, AreaEntityWorldSnapshot, {
    free_prefix => plain_vec(free_prefix),
    sectors => plain_vec(sectors),
    entities => plain_vec(entities),
}; world_mid, world_half, contiguous_free_head);

account!(kernel, EntityKernelSnapshot, {
    slots => plain_vec(slots),
    free_fifo => plain_vec(free_fifo),
}; high_water, level_time_ms, frame_serial);

account!(event, EventRecord, {
    audience => self::audience(audience),
}; sequence, tick, event);

account!(entity_event, EntityEventRecord, {
    audience => self::audience(audience),
}; sequence, tick, event, payload);

account!(shock, hud_iw4::ShockParams, {
    sound => shock_sound(sound),
}; screen_type, blur_blend_ms, blur_fade_ms, white_fade_ms, shot_fade_ms, look, movement);

account!(shock_sound, hud_iw4::ShellshockSoundParms, {
    channel_volumes => channel_volumes.as_ref().map_or(0, |map| map.iter().fold(0usize, |n, (key, _)| n.saturating_add(1024).saturating_add(key.capacity()))),
    loop_alias => loop_alias.capacity(),
    end_alias => end_alias.capacity(),
    abort_alias => abort_alias.capacity(),
}; affect);

account!(dobj, xmodel_runtime::DObjSemanticState, {
    composition => self::composition(composition),
    tree => tree.as_ref().map_or(0, self::tree),
}; pose_revision, requested_parts, hide_part_bits);

account!(composition, xmodel_runtime::DObjCompositionDescriptor, {
    models => nested_vec(models, model),
}; revision);

account!(model, xmodel_runtime::DObjModelDescriptor, {
    model => model.capacity(),
    attach_tag => attach_tag.as_ref().map_or(0, String::capacity),
}; parent_model, ignore_collision);

account!(tree, xmodel_runtime::XAnimTreeSnapshot, {
    nodes => nested_vec(nodes, node),
}; definition_revision, state_revision);

account!(node, xmodel_runtime::XAnimSemanticNode, {
    clip => clip.as_ref().map_or(0, String::capacity),
}; parent, kind, parts, state);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_and_spare_capacity_are_counted_once() {
        let mut s = Snapshot::unpublished(Tick(0));
        assert_eq!(s.owned_bytes(), size_of::<Snapshot>());
        s.players.reserve_exact(7);
        assert_eq!(
            s.owned_bytes(),
            size_of::<Snapshot>()
                + s.players.capacity() * size_of::<(ClientId, playerstate_iw4::PlayerState)>()
        );
    }

    #[test]
    fn nested_client_and_objective_strings_grow_the_estimate() {
        let mut s = Snapshot::unpublished(Tick(0));
        s.meta
            .clients
            .push((ClientId(1), ClientSnapshotMeta::default()));
        let before = s.owned_bytes();
        s.meta.clients[0]
            .1
            .client_dvars
            .push(("key".into(), "v".repeat(8192)));
        s.meta
            .objectives
            .server_info
            .push(("info".into(), "v".repeat(8192)));
        assert!(s.owned_bytes() >= before + 16384);
        let cloned = s.clone();
        assert_eq!(cloned, s);
        assert!(cloned.owned_bytes() >= before + 16384);
    }

    #[test]
    fn semantic_models_and_animation_clip_names_are_counted() {
        let state = xmodel_runtime::DObjSemanticState::one_leaf(
            "model".repeat(1024),
            "clip".repeat(1024),
            1,
            1,
            0.0,
        );
        assert!(dobj(&state) >= 9216);
    }

    #[test]
    fn tree_map_allowance_and_keys_are_included() {
        let mut sound = hud_iw4::ShellshockSoundParms {
            affect: true,
            channel_volumes: None,
            loop_alias: String::new(),
            end_alias: String::new(),
            abort_alias: String::new(),
        };
        let before = shock_sound(&sound);
        let mut volumes = std::collections::BTreeMap::new();
        volumes.insert("channel".repeat(1024), 0.5);
        sound.channel_volumes = Some(volumes);
        assert!(shock_sound(&sound) >= before + 1024 + 7168);
    }
}
