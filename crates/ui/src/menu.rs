use bevy::prelude::*;
use frame::ClientSet;

use crate::classes::store::{
    ClassStoreFile, load_class_store, save_class_store, sync_host_class_loadouts,
};

#[derive(Resource, Clone, Debug, Default)]
pub struct MenuMapList(pub Vec<asset_transport::MapPack>);

impl MenuMapList {
    pub fn maps(&self) -> impl Iterator<Item = &String> {
        self.0.iter().flat_map(|pack| &pack.maps)
    }

    pub fn contains(&self, map: &str) -> bool {
        self.maps().any(|installed| installed == map)
    }

    pub fn pack_of(&self, map: &str) -> Option<usize> {
        self.0
            .iter()
            .position(|pack| pack.maps.iter().any(|installed| installed == map))
    }
}

pub(crate) struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuMapList>()
            .init_resource::<crate::barracks::BarracksProfile>()
            .init_resource::<crate::ClassLoadoutCatalog>()
            .init_resource::<frame::GameSettings>()
            .init_resource::<crate::BindingView>()
            .init_resource::<crate::SessionClassStore>()
            .init_resource::<ClassStoreFile>()
            .init_resource::<frame::HostClassLoadouts>()
            .init_resource::<asset_game::MenuCatalog>()
            .init_resource::<asset_game::LocalizeCatalog>()
            .add_systems(
                Update,
                (
                    crate::options::apply_window_settings,
                    load_class_store,
                    crate::barracks::load_profile,
                    crate::barracks::save_profile,
                    sync_host_class_loadouts,
                    save_class_store,
                )
                    .chain()
                    .in_set(ClientSet::Ui),
            );
    }
}

pub fn install_frontend_menus(catalog: &mut asset_game::MenuCatalog) -> Result<(), String> {
    catalog.load_definitions(include_str!("../menus/frontend.json"))?;
    catalog.load_definitions(include_str!("../menus/connection_error.json"))?;
    catalog.load_definitions(include_str!("../menus/map_error.json"))?;
    catalog.load_definitions(include_str!("../menus/classes.json"))?;
    catalog.load_definitions(include_str!("../menus/barracks.json"))?;
    catalog.load_definitions(include_str!("../menus/settings.json"))?;
    catalog.load_definitions(include_str!("../menus/controller.json"))?;
    catalog.load_definitions(include_str!("../menus/game_folders.json"))?;
    catalog.load_definitions(include_str!("../menus/pocket.json"))?;
    let slider = catalog
        .get("pc_options_video")
        .and_then(|menu| {
            menu.items
                .iter()
                .find(|item| item.name == "video_brightness")
        })
        .cloned();
    for (name, menu) in &mut catalog.menus {
        if name == "pc_options_look"
            && let Some(template) = &slider
            && let Some(y) = menu
                .items
                .iter()
                .find(|item| item.text_key == "@MENU_MOUSE_SENSITIVITY")
                .map(|label| label.rect.y)
            && let Some(item) = menu
                .items
                .iter_mut()
                .find(|item| item.item_type == asset_game::ITEM_TYPE_SLIDER)
        {
            *item = asset_game::MenuItem {
                name: "look_sensitivity".into(),
                dvar: "ui_sensitivity".into(),
                slider: Some(asset_game::MenuSlider {
                    min: 0.1,
                    max: 30.0,
                    step: 0.1,
                    display_range: None,
                    decimals: 1,
                    suffix: String::new(),
                }),
                ..template.clone()
            };
            item.rect.y = y;
        }
        add_bot_rows(name, menu);
        if matches!(name.as_str(), "popup_endgame" | "popup_endgame_ranked") {
            for item in &mut menu.items {
                if item.name == "button_yes" {
                    // MW2's own response: the host ends the match through forceEnd,
                    // so it ends as a match end. The hud turns it into a disconnect
                    // for anyone but the host of a private lobby.
                    item.handlers.action = vec![asset_game::MenuEvent::Script(
                        "play mouse_click; close self; scriptmenuresponse \"endround\";".into(),
                    )];
                }
            }
        }
        if let Some(settings_link) = menu
            .items
            .iter()
            .find(|item| {
                item.item_type == 1
                    && matches!(item.text_key.as_str(), "@MENU_CHAT" | "@MENU_VOICE")
            })
            .cloned()
            && menu
                .items
                .iter()
                .any(|item| item.text_key == "@MENU_RESET_SYSTEM_DEFAULTS")
        {
            let mut multiplayer = settings_link;
            multiplayer.name = "multiplayer_settings".into();
            multiplayer.text_key = "@MENU_MULTIPLAYER_OPTIONS".into();
            multiplayer.rect.y = 88.0;
            multiplayer.vis_exp = "1".into();
            multiplayer.disabled_exp = "0".into();
            multiplayer.handlers.action = vec![asset_game::MenuEvent::Script(
                "play mouse_click; close self; open options_multi;".into(),
            )];
            let mut controller = multiplayer.clone();
            let mut game_folders = multiplayer.clone();
            let mut pocket = multiplayer.clone();
            menu.items.push(multiplayer);
            controller.name = "controller_settings".into();
            controller.text_key = "Controller".into();
            controller.rect.y = 108.0;
            controller.handlers.action = vec![asset_game::MenuEvent::Script(
                "play mouse_click; close self; open options_controller;".into(),
            )];
            menu.items.push(controller);
            game_folders.name = "game_folders_settings".into();
            game_folders.text_key = "Game Folders".into();
            game_folders.rect.y = 128.0;
            game_folders.handlers.action = vec![asset_game::MenuEvent::Script(
                "play mouse_click; close self; open options_game_folders;".into(),
            )];
            menu.items.push(game_folders);
            pocket.name = "pocket_settings".into();
            pocket.text_key = "IW4 Pocket options".into();
            pocket.rect.y = 148.0;
            pocket.handlers.action = vec![asset_game::MenuEvent::Script(
                "play mouse_click; close self; open options_pocket;".into(),
            )];
            menu.items.push(pocket);
        }

        let removed_rows: Vec<_> = menu
            .items
            .iter()
            .filter(|item| {
                item.item_type == 1
                    && matches!(
                        item.text_key.as_str(),
                        "@MENU_VOICE" | "@MENU_CHAT" | "@MENU_RESET_SYSTEM_DEFAULTS"
                    )
            })
            .map(|item| (item.rect.x, item.rect.y))
            .collect();
        menu.items.retain(|item| {
            !removed_rows.iter().any(|&(x, y)| {
                item.rect.x == x
                    && item.rect.y == y
                    && item.name != "multiplayer_settings"
                    && item.name != "controller_settings"
                    && item.name != "game_folders_settings"
                    && item.name != "pocket_settings"
            })
        });
        if name == "pc_options_controls" {
            for item in &mut menu.items {
                if item.rect.x >= 232.0 && item.rect.y > 88.0 {
                    item.rect.y -= 20.0;
                }
            }
        }

        let Some(mode) = name.strip_prefix("settings_quick_") else {
            continue;
        };
        if sim::HostGameModeSelection::from_token(mode).is_none() {
            continue;
        }
        for item in &mut menu.items {
            if item.dvar == "camera_thirdperson" {
                if item.item_type == 12 {
                    item.choices.clear();
                    item.dvar.clear();
                    item.text_key = "Недоступно".into();
                }
                item.disabled_exp = "1".into();
                item.item_type = 0;
                item.static_flags |= 0x100000;
                item.handlers.action.clear();
                item.fore_color[3] = 0.4;
            } else if item.item_type == 1 && !item.dvar.is_empty() {
                item.text_scale = 0.30;
            }
        }
    }
    Ok(())
}


/// The in-game "Options" menu (Choose Class, Change Team, Options, Mute Players, End Game)
/// comes from the game's own files. It gets two extra rows, "Add 1 bot" and "Add 5 bots",
/// but only when the menu has the Mute and End Game rows, so no other menu is touched.
/// The rows run `bot add` only on the host (`sv_running`); everyone else sees them do nothing.
/// Every menu that looks like a candidate is named in the log, so a wrong guess shows up
/// there instead of as a broken menu.
fn add_bot_rows(name: &str, menu: &mut asset_game::MenuDef) {
    let key = |item: &asset_game::MenuItem| item.text_key.to_ascii_uppercase();
    let rows: Vec<&asset_game::MenuItem> = menu.items.iter().filter(|item| item.item_type == 1).collect();
    let mute = rows.iter().find(|item| key(item).contains("MUTE"));
    let end = rows
        .iter()
        .any(|item| key(item).contains("END_GAME") || key(item).contains("ENDGAME") || key(item).contains("LEAVE"));
    if mute.is_none() && !end {
        return;
    }
    let keys: Vec<String> = rows.iter().map(|item| item.text_key.clone()).collect();
    diag::info!(Ui, "menu candidate for bot rows: {name} rows={keys:?}");
    let (Some(template), true) = (mute.map(|item| (*item).clone()), end) else {
        return;
    };
    if menu.items.iter().any(|item| item.name == "pocket_bot_add_1") {
        return;
    }
    let mut ys: Vec<f32> = rows.iter().map(|item| item.rect.y).collect();
    ys.sort_by(|a, b| a.total_cmp(b));
    let last = ys.last().copied().unwrap_or(0.0);
    let gap = ys
        .windows(2)
        .map(|w| w[1] - w[0])
        .filter(|g| *g > 1.0)
        .fold(f32::MAX, f32::min);
    let gap = if gap == f32::MAX { 20.0 } else { gap };
    for (at, (label, count)) in [("Add 1 bot", 1), ("Add 5 bots", 5)].into_iter().enumerate() {
        let mut row = template.clone();
        row.name = format!("pocket_bot_add_{count}");
        row.text_key = label.into();
        row.rect.y = last + gap * (at as f32 + 1.0);
        row.vis_exp = "1".into();
        row.disabled_exp = "0".into();
        row.handlers.action = vec![asset_game::MenuEvent::Script(format!(
            "play mouse_click; if (getDvar \"sv_running\" == \"1\") exec \"bot add {count}\";"
        ))];
        menu.items.push(row);
    }
    diag::info!(Ui, "menu {name}: added bot rows below y={last}");
}
