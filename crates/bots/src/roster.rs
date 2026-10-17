use bevy::prelude::Resource;
use sim::{ClientId, LifeSequence};

use crate::controller::HostController;

pub const MAX_HOST_BOTS: u32 = 20;

const BOT_NAMES: &[&str] = &[
    "xXsn1perXx",
    "xX_QuickScope_X",
    "xXNoScopeXx",
    "Xx360NoScopexX",
    "xXDarkLord69Xx",
    "xXPwnageXx",
    "xXMLG_ProXx",
    "xXTrickshotXx",
    "xXHeadshotXx",
    "xXDeadeyeXx",
    "xXShadowXx",
    "xXKillaXx",
    "ImbaPwnz",
    "Mt_Dew_Dad",
    "DoritoDan",
    "Snip3rWolf",
    "FaZe_Bob",
    "OptiMLG",
    "MLGsnoopy",
    "iHaxPro",
    "Ligma",
    "SugmaBalls",
    "Sawcondese",
    "Joe_Mama",
    "Bofa_Dad",
    "Deez_Nuts",
    "ImBadAtThis",
    "Yeet",
    "GetRekt",
    "Rekt_u",
    "420BlazeIt",
    "xX420Xx",
    "WeedLord",
    "NoobSlayer",
    "n00bTube",
    "Pwn3d",
    "KnifeOnly",
    "NoSc0pe",
    "OneShotOne",
    "Hardscoper",
    "AimbotAndy",
    "NotAHacker",
    "ModdedLobby",
    "RIP_Rust",
    "HaloReach",
    "Mom_Get_Cam",
    "TheBigMan",
    "ChairBreaker",
    "Rage_Quit",
    "Lag_Switch",
    "QuickyBoy",
    "RustPwner",
    "Sniper_Boi",
    "ProGamerMove",
    "GG_EZ",
    "ez_clap",
    "OopsDied",
    "WheresMyTeam",
    "Camper_Carl",
    "AFK_Andy",
    "ChatGPTDeezNuts",
    "OnlyRAMs",
    "CtrlAltDefeat",
    "AltF4YourMom",
    "404NoBitches",
    "StackOverHoe",
    "SudoEatMyAss",
    "BlueScreenBalls",
    "CallOfBooty",
    "CumAndConquer",
    "WankOfWarcraft",
    "AssCreed",
    "RedDeadErection",
    "EldenSchlong",
    "NoScopeNoBitches",
    "PrematureReload",
    "TacticalTeabag",
    "CampAndCope",
    "ObiWanCanBlowMe",
    "DarthVibrator",
    "HarryPooter",
    "LordOfTheDings",
    "ShrekOnMyFace",
    "WalterWank",
    "BetterCallBalls",
    "GandalfTheGay",
    "FrodoTeaBaggins",
    "WillyWanker",
    "MrLeast",
    "BeastDeezNuts",
    "SubscribeOrDie",
    "RaidShadowBalls",
    "NordVPNis",
    "FyreFestVIP",
    "CryptoRugDaddy",
    "OnlyScams",
    "ApologyUkulele",
    "ColossalSkillIssue",
    "SamAltF4",
    "ClippyGotHard",
    "DeepSeekDeez",
    "GrokTuah",
    "GitPushMyAss",
    "HotfixMyBalls",
    "SeggsFault",
    "AcceptMyCookies",
    "MicroSoftDick",
    "RebootyCall",
    "DukeNutted",
    "MasterQueef",
    "SolidSnakeOil",
    "MetalGearSuck",
    "AssEffect",
    "GirthOfWar",
    "MortalCumback",
    "FapSouls",
    "SekiroNoBitches",
    "Battlefield69",
    "PayToWhine",
    "FlashbangMyAss",
    "CountFapula",
    "Dumbledong",
    "HagridGotHard",
    "Voldemortgage",
    "ThanosDidHalf",
    "TheWankingDead",
    "BreakingBalls",
    "StrangerDongs",
    "JohnWank",
    "JackOffSparrow",
    "HawkTuah404",
    "MrBreast",
    "PewDiePieChart",
    "iShowDebt",
    "SkibidiTaxFraud",
    "SigmaBalls",
    "RizzInPeace",
    "SponsoredByDebt",
    "TrustMeBroCoin",
    "LinkInMyBussy",
    "YourMomsDuo",
    "MyWifeLeft",
    "DivorcedByPing",
    "BornToRespawn",
    "UnpaidHitman",
    "SendDudes",
    "TeabagReceipt",
    "OopsWrongHole",
];

// The protocol stores 16 bytes including NUL. All approved names are ASCII.
// Keep the full pool above, but use the same explicit 15-byte display name in
// the lobby and authoritative game state (including the two longer names).
const BOT_NAME_BYTES: usize = 15;

fn display_name(name: &'static str) -> &'static str {
    &name[..name.len().min(BOT_NAME_BYTES)]
}

fn shuffle_names(names: &mut [&'static str], mut random: impl FnMut() -> u64) {
    for i in (1..names.len()).rev() {
        // Rejection sampling avoids modulo bias.
        let bound = (i + 1) as u64;
        let threshold = bound.wrapping_neg() % bound;
        let value = loop {
            let value = random();
            if value >= threshold {
                break value;
            }
        };
        names.swap(i, (value % bound) as usize);
    }
}

fn match_names() -> &'static [&'static str] {
    static NAMES: std::sync::OnceLock<Vec<&'static str>> = std::sync::OnceLock::new();
    NAMES.get_or_init(|| {
        let mut names: Vec<_> = BOT_NAMES.iter().map(|name| display_name(name)).collect();
        let mut entropy = [0u8; 8];
        if let Err(error) = getrandom::fill(&mut entropy) {
            diag::warn!(
                Sim,
                "bots: name shuffle entropy unavailable: {error}; using stable fallback"
            );
            return names;
        }
        let mut state = u64::from_le_bytes(entropy);
        shuffle_names(&mut names, || {
            // SplitMix64: independent from bot AI and account identity RNGs.
            state = state.wrapping_add(0x9e3779b97f4a7c15);
            let mut z = state;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
            z ^ (z >> 31)
        });
        names
    })
}

/// Random order once per app session, stable across lobby refresh and map load.
pub fn lobby_bot_name(index: u32) -> &'static str {
    let names = match_names();
    names[index as usize % names.len()]
}

#[derive(Resource, Debug, Default)]
pub struct BotAddQueue(pub Vec<BotAddRequest>);

#[derive(Debug)]
pub struct BotAddRequest {
    pub count: u32,
    pub dummy: bool,
}

impl BotAddQueue {
    pub fn push(&mut self, count: u32) {
        self.0.push(BotAddRequest {
            count: count.max(1),
            dummy: false,
        });
    }

    pub fn push_dummy(&mut self, count: u32) {
        self.0.push(BotAddRequest {
            count: count.max(1),
            dummy: true,
        });
    }

    pub fn drain(&mut self) -> Vec<BotAddRequest> {
        core::mem::take(&mut self.0)
    }
}

#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct BotHold(pub bool);

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BotTpTarget {
    All,
    Id(ClientId),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BotTpWhere {
    Absolute {
        origin: [f32; 3],
        yaw: Option<f32>,
        pitch: Option<f32>,
    },

    Above {
        height: f32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BotTpRequest {
    pub target: BotTpTarget,
    pub where_: BotTpWhere,
}

#[derive(Resource, Debug, Default)]
pub struct BotTpQueue(pub Vec<BotTpRequest>);

impl BotTpQueue {
    pub fn push(&mut self, request: BotTpRequest) {
        self.0.push(request);
    }

    pub fn drain(&mut self) -> Vec<BotTpRequest> {
        core::mem::take(&mut self.0)
    }
}

#[derive(Resource, Debug, Default)]
pub struct BotFireQueue(pub Vec<BotTpTarget>);

impl BotFireQueue {
    pub fn push(&mut self, target: BotTpTarget) {
        self.0.push(target);
    }

    pub fn drain(&mut self) -> Vec<BotTpTarget> {
        core::mem::take(&mut self.0)
    }
}

#[derive(Debug)]
pub struct BotSlot {
    pub id: ClientId,
    pub name: &'static str,
    pub account: sim::AccountId,
    pub brain: Option<HostController>,
    pub joined: bool,
    pub class_picked_in: Option<LifeSequence>,
    pub class_picks: u32,
}

#[derive(Resource, Debug)]
pub struct BotRoster {
    pub bots: Vec<BotSlot>,
    pub next_client: u32,
    pub seed: u64,
}

impl Default for BotRoster {
    fn default() -> Self {
        Self {
            bots: Vec::new(),

            next_client: 1,
            seed: 0xb075_0001,
        }
    }
}

impl BotRoster {
    pub fn is_bot(&self, id: ClientId) -> bool {
        self.bots.iter().any(|bot| bot.id == id)
    }

    // `taken` are the ids real clients already own. A bot minted onto one of
    // them *is* that client as far as the roster is concerned: `is_bot` claims
    // the player, and the slot is dead weight because no system can drive an
    // id someone else is already playing.
    pub fn add_bots(&mut self, count: u32, taken: &[ClientId], dummy: bool) -> Vec<ClientId> {
        let room = MAX_HOST_BOTS.saturating_sub(self.bots.len() as u32);
        let count = count.min(room);
        let seed = self.seed;
        let mut added = Vec::with_capacity(count as usize);
        for i in 0..count {
            let Some(id) = self.claim_id(taken) else {
                break;
            };
            let mut account = [0; 16];
            while account == [0; 16] {
                if let Err(error) = getrandom::fill(&mut account) {
                    diag::warn!(Sim, "bots: account identity creation failed: {error}");
                    return added;
                }
            }
            let brain = (!dummy)
                .then(|| HostController::new(seed ^ (u64::from(id.0) << 32) ^ u64::from(i)));
            let available: Vec<_> = match_names()
                .iter()
                .copied()
                .filter(|name| !self.bots.iter().any(|bot| bot.name == *name))
                .collect();
            // Same session order as the lobby, skipping bots already present.
            let name = available[0];
            self.bots.push(BotSlot {
                id,
                name,
                account: sim::AccountId(account),
                brain,
                joined: false,
                class_picked_in: None,
                class_picks: 0,
            });
            added.push(id);
        }
        added
    }

    fn claim_id(&mut self, taken: &[ClientId]) -> Option<ClientId> {
        // Only `bots + taken` ids are spoken for, so one candidate more than
        // that always turns up a free one.
        let candidates = self
            .bots
            .len()
            .saturating_add(taken.len())
            .saturating_add(1);
        for _ in 0..candidates {
            let id = ClientId(self.next_client);
            self.next_client = self.next_client.wrapping_add(1).max(1);
            if !taken.contains(&id) && !self.is_bot(id) {
                return Some(id);
            }
        }
        None
    }
}

pub fn default_class_index(seed: u64, client: ClientId, pick: u32) -> u8 {
    let mix = seed
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(u64::from(client.0).wrapping_mul(0xBF58_476D_1CE4_E5B9))
        .wrapping_add(u64::from(pick).wrapping_mul(0x94D0_49BB_1331_11EB));
    (mix >> 33) as u8
}

#[cfg(test)]
mod name_tests {
    use super::*;

    #[test]
    fn pool_is_ascii_unique_and_protocol_safe() {
        assert_eq!(BOT_NAMES.len(), 148);
        let mut visible = std::collections::HashSet::new();
        for &name in BOT_NAMES {
            assert!(name.is_ascii());
            let shown = display_name(name);
            assert!(shown.len() <= BOT_NAME_BYTES);
            assert!(visible.insert(shown), "duplicate display name: {shown}");
            let packed = entity_iw4::pack_client_state_name(shown);
            assert_eq!(&packed[..shown.len()], shown.as_bytes());
            assert_eq!(packed[shown.len()], 0);
        }
        assert!(BOT_NAMES.contains(&"GandalfTheGay"));
        assert_eq!(display_name("NoScopeNoBitches"), "NoScopeNoBitche");
    }

    #[test]
    fn shuffle_is_permutation_not_fixed_order() {
        let mut names = BOT_NAMES.to_vec();
        shuffle_names(&mut names, || u64::MAX);
        assert_ne!(names, BOT_NAMES);
        names.sort_unstable();
        let mut expected = BOT_NAMES.to_vec();
        expected.sort_unstable();
        assert_eq!(names, expected);
    }

    #[test]
    fn lobby_and_roster_share_stable_names() {
        let expected: Vec<_> = (0..MAX_HOST_BOTS).map(lobby_bot_name).collect();
        let mut roster = BotRoster::default();
        roster.add_bots(5, &[], true);
        roster.add_bots(MAX_HOST_BOTS - 5, &[], true);
        assert_eq!(
            roster.bots.iter().map(|b| b.name).collect::<Vec<_>>(),
            expected
        );
        assert_eq!(
            (0..MAX_HOST_BOTS).map(lobby_bot_name).collect::<Vec<_>>(),
            expected
        );
    }
}
