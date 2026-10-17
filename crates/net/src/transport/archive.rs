use std::collections::VecDeque;

use bevy::prelude::Resource;
use sim::{ClientId, ClientLifecycle, Snapshot, Tick};

use crate::authority::inbox::AUTHORITY_MS;

pub const ARCHIVE_TICK_MS: i32 = AUTHORITY_MS;

pub const ARCHIVE_CACHED_SNAPSHOT_CLIENTS: usize = 0x144;

pub const ARCHIVE_MAX_TICKS: usize = 0x4b0;

pub const ARCHIVE_BYTE_BUDGET: usize = 32 * 1024 * 1024;

pub const ARCHIVE_LOOKUP_WINDOW_TICKS: usize = ARCHIVE_MAX_TICKS;

pub const fn archive_attainable_ticks(_clients: usize) -> usize {
    ARCHIVE_MAX_TICKS
}

#[derive(Clone, Debug, PartialEq)]
pub struct ArchivedFrame {
    pub tick: Tick,
    pub snapshot: Snapshot,

    pub no_player_state: Vec<ClientId>,
}

impl ArchivedFrame {
    pub fn records(&self) -> usize {
        self.snapshot.players.len()
    }

    pub fn player_state_exists(&self, client: ClientId) -> bool {
        !self.no_player_state.contains(&client)
            && self.snapshot.players.iter().any(|(id, _)| *id == client)
    }

    pub fn bytes(&self) -> usize {
        self.snapshot
            .owned_bytes()
            .saturating_add(core::mem::size_of::<Self>() - core::mem::size_of::<Snapshot>())
            .saturating_add(self.no_player_state.capacity() * core::mem::size_of::<ClientId>())
    }
}

// Box keeps unused deque slots small. Cache the payload estimate once on insert.
#[derive(Clone, Debug)]
struct StoredFrame {
    frame: ArchivedFrame,
    bytes: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArchiveLookup {
    pub attained_ms: i32,

    pub tick: Option<Tick>,
}

impl ArchiveLookup {
    pub fn nothing_to_show(&self) -> bool {
        self.tick.is_none()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ArchiveMetrics {
    pub archived: u64,

    pub evicted: u64,

    pub peak_ticks: usize,

    pub peak_records: usize,

    pub peak_bytes: usize,

    pub oversized: u64,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct FrameArchive {
    frames: VecDeque<Box<StoredFrame>>,
    records: usize,
    bytes: usize,
    metrics: ArchiveMetrics,
    report_ticks: u32,
}

impl FrameArchive {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, frame: ArchivedFrame) -> bool {
        if !self.accepts_tick(frame.tick) {
            return false;
        }
        let bytes = frame.bytes().saturating_add(core::mem::size_of::<usize>());
        if !self.make_room(bytes) {
            return false;
        }
        self.records += frame.records();
        self.bytes += bytes;
        self.frames
            .push_back(Box::new(StoredFrame { frame, bytes }));
        self.metrics.archived = self.metrics.archived.saturating_add(1);
        self.metrics.peak_ticks = self.metrics.peak_ticks.max(self.frames.len());
        self.metrics.peak_records = self.metrics.peak_records.max(self.records);
        self.metrics.peak_bytes = self.metrics.peak_bytes.max(self.bytes());
        true
    }

    fn accepts_tick(&self, tick: Tick) -> bool {
        self.newest().is_none_or(|newest| tick.0 > newest.tick.0)
    }

    pub fn push_snapshot(&mut self, snapshot: &Snapshot, spectators: &[ClientId]) -> bool {
        // Reject duplicate ticks before any deep clone or metadata traversal.
        if !self.accepts_tick(snapshot.tick) {
            return false;
        }
        let no_player_state: Vec<_> = snapshot
            .meta
            .clients
            .iter()
            .filter(|(id, meta)| {
                spectators.contains(id)
                    || !matches!(
                        meta.lifecycle,
                        ClientLifecycle::Alive
                            | ClientLifecycle::Dead
                            | ClientLifecycle::RespawnPending
                    )
            })
            .map(|(id, _)| *id)
            .collect();
        // Source capacities conservatively preflight the clone. Evict first to
        // avoid retaining a full archive plus a new snapshot transiently.
        let estimate = snapshot
            .owned_bytes()
            .saturating_add(core::mem::size_of::<StoredFrame>() - core::mem::size_of::<Snapshot>())
            .saturating_add(no_player_state.capacity() * core::mem::size_of::<ClientId>());
        let accepted = self.make_room(estimate)
            && self.push(ArchivedFrame {
                tick: snapshot.tick,
                snapshot: snapshot.clone(),
                no_player_state,
            });
        self.report_ticks = self.report_ticks.wrapping_add(1);
        if self.report_ticks == 1 || self.report_ticks % 200 == 0 {
            self.report("sample");
        }
        accepted
    }

    fn make_room(&mut self, incoming: usize) -> bool {
        // Reserve only pointer slots; their actual capacity is part of the budget.
        if self.frames.capacity() == 0 {
            self.frames.reserve_exact(ARCHIVE_MAX_TICKS);
        }
        if incoming.saturating_add(self.container_bytes()) > ARCHIVE_BYTE_BUDGET {
            self.metrics.oversized = self.metrics.oversized.saturating_add(1);
            return false; // Never retain an unbounded single frame.
        }
        while self.frames.len() >= ARCHIVE_MAX_TICKS
            || self.bytes().saturating_add(incoming) > ARCHIVE_BYTE_BUDGET
        {
            let Some(oldest) = self.frames.pop_front() else {
                break;
            };
            self.records -= oldest.frame.records();
            self.bytes -= oldest.bytes;
            self.metrics.evicted = self.metrics.evicted.saturating_add(1);
        }
        true
    }

    fn container_bytes(&self) -> usize {
        self.frames.capacity() * core::mem::size_of::<Box<StoredFrame>>()
    }

    fn report(&self, reason: &str) {
        diag::info!(
            Net,
            "frame_archive: {reason} frames={} records={} accounted_bytes={} peak_bytes={} evictions={} oversized={} retained_ms={} oldest_tick={:?} newest_tick={:?} (payload estimate, not allocator bytes){}",
            self.len(),
            self.records(),
            self.bytes(),
            self.metrics.peak_bytes,
            self.metrics.evicted,
            self.metrics.oversized,
            self.duration_ms(),
            self.oldest().map(|f| f.tick.0),
            self.newest().map(|f| f.tick.0),
            diag::ios_env::event_memory_tag()
        );
    }

    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn records(&self) -> usize {
        self.records
    }

    pub fn bytes(&self) -> usize {
        self.bytes.saturating_add(self.container_bytes())
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn metrics(&self) -> ArchiveMetrics {
        self.metrics
    }

    pub fn oldest(&self) -> Option<&ArchivedFrame> {
        self.frames.front().map(|stored| &stored.frame)
    }

    pub fn newest(&self) -> Option<&ArchivedFrame> {
        self.frames.back().map(|stored| &stored.frame)
    }

    pub fn frame(&self, tick: Tick) -> Option<&ArchivedFrame> {
        self.frames
            .iter()
            .rev()
            .map(|s| &s.frame)
            .find(|f| f.tick == tick)
    }

    fn frame_count(&self) -> Option<i64> {
        self.newest().map(|f| i64::from(f.tick.0) + 1)
    }

    pub fn duration_ms(&self) -> i32 {
        let (Some(count), Some(oldest)) = (self.frame_count(), self.oldest()) else {
            return 0;
        };
        ((count - i64::from(oldest.tick.0)) * i64::from(ARCHIVE_TICK_MS)) as i32
    }

    pub fn lookup(&self, requested_ms: i32) -> ArchiveLookup {
        if requested_ms < 1 {
            return ArchiveLookup {
                attained_ms: requested_ms,
                tick: None,
            };
        }
        let (Some(count), Some(oldest)) = (self.frame_count(), self.oldest()) else {
            return ArchiveLookup {
                attained_ms: 0,
                tick: None,
            };
        };
        let back = i64::from(requested_ms) / i64::from(ARCHIVE_TICK_MS);
        let index = (count - back).max(i64::from(oldest.tick.0));
        match self
            .frames
            .iter()
            .map(|s| &s.frame)
            .find(|f| i64::from(f.tick.0) >= index)
        {
            Some(found) => ArchiveLookup {
                attained_ms: ((count - i64::from(found.tick.0)) * i64::from(ARCHIVE_TICK_MS))
                    as i32,
                tick: Some(found.tick),
            },

            None => ArchiveLookup {
                attained_ms: 0,
                tick: None,
            },
        }
    }

    pub fn clear(&mut self) {
        self.report("clear");
        self.frames = VecDeque::new();
        self.report_ticks = 0;
        self.records = 0;
        self.bytes = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot(tick: u32, metadata_bytes: usize) -> Snapshot {
        let mut s = Snapshot::unpublished(Tick(tick));
        if metadata_bytes > 0 {
            s.meta.hud_strings.push((0, "x".repeat(metadata_bytes)));
        }
        s
    }

    #[test]
    fn omitted_metadata_is_counted_and_retained_unchanged() {
        let small = snapshot(1, 0);
        let large = snapshot(1, 1024 * 1024);
        assert!(large.owned_bytes() >= small.owned_bytes() + 1024 * 1024);
        let mut archive = FrameArchive::new();
        assert!(archive.push_snapshot(&large, &[]));
        assert_eq!(archive.newest().unwrap().snapshot, large);
        assert!(archive.bytes() >= large.owned_bytes());
    }

    #[test]
    fn byte_budget_evicts_and_lookup_clamps_to_retained_history() {
        let mut archive = FrameArchive::new();
        for tick in 1..=40 {
            assert!(archive.push_snapshot(&snapshot(tick, 1024 * 1024), &[]));
            assert!(archive.bytes() <= ARCHIVE_BYTE_BUDGET);
            assert!(archive.len() <= ARCHIVE_MAX_TICKS);
        }
        assert!(archive.metrics().evicted > 0);
        let oldest = archive.oldest().unwrap().tick;
        assert!(oldest.0 > 1);
        assert!(archive.frame(Tick(1)).is_none());
        let lookup = archive.lookup(60_000);
        assert_eq!(lookup.tick, Some(oldest));
        assert_eq!(lookup.attained_ms, archive.duration_ms());
        assert_eq!(archive.frame(Tick(40)).unwrap().snapshot.tick, Tick(40));
        assert!(archive.lookup(0).nothing_to_show());
    }

    #[test]
    fn oversized_frame_is_rejected_without_losing_existing_history() {
        let mut archive = FrameArchive::new();
        assert!(archive.push_snapshot(&snapshot(1, 0), &[]));
        let before = archive.bytes();
        let huge = snapshot(2, ARCHIVE_BYTE_BUDGET);
        assert!(!archive.push_snapshot(&huge, &[]));
        assert_eq!(archive.len(), 1);
        assert_eq!(archive.bytes(), before);
        assert_eq!(archive.metrics().oversized, 1);
        assert_eq!(archive.newest().unwrap().tick, Tick(1));
    }

    #[test]
    fn duplicate_and_old_ticks_do_not_evict_or_change_metrics() {
        let mut archive = FrameArchive::new();
        assert!(archive.push_snapshot(&snapshot(2, 0), &[]));
        let metrics = archive.metrics();
        let bytes = archive.bytes();
        assert!(!archive.push_snapshot(&snapshot(2, 0), &[]));
        assert!(!archive.push_snapshot(&snapshot(1, 0), &[]));
        assert_eq!(archive.metrics(), metrics);
        assert_eq!(archive.bytes(), bytes);
    }

    #[test]
    fn tick_bound_clear_and_reuse() {
        let mut archive = FrameArchive::new();
        for tick in 1..=(ARCHIVE_MAX_TICKS as u32 + 1) {
            assert!(archive.push_snapshot(&snapshot(tick, 0), &[]));
            assert!(archive.len() <= ARCHIVE_MAX_TICKS);
            assert!(archive.bytes() <= ARCHIVE_BYTE_BUDGET);
        }
        assert!(archive.metrics().evicted > 0);
        archive.clear();
        assert!(archive.is_empty());
        assert_eq!(archive.bytes(), 0);
        assert_eq!(archive.records(), 0);
        assert!(archive.lookup(50).nothing_to_show());
        assert!(archive.push_snapshot(&snapshot(1, 0), &[]));
    }

    #[test]
    fn killcam_focus_clamps_after_byte_eviction_and_handles_absent_focus() {
        let mut archive = FrameArchive::new();
        for tick in 1..=40 {
            let mut s = snapshot(tick, 1024 * 1024);
            s.players.push((ClientId(7), playerstate_iw4::PlayerState::ZERO));
            assert!(archive.push_snapshot(&s, &[]));
        }
        let mut session = crate::policy::seat::KillcamSession {
            archivetime_ms: 60_000,
            focus_client: ClientId(7),
            focus: playerstate_iw4::SeatFocus::hitscan(7),
            ends_at_ms: i32::MAX,
            final_kill: false,
            started_at_ms: 0,
            killcamoffset_ms: 60_000,
            predelay_ms: 0,
            postdelay_ms: 0,
            recalc_pending: false,
            entity_focus: None,
        };
        crate::policy::killcam::follow_archived_focus(&archive, &mut session);
        assert!(archive.oldest().unwrap().tick.0 > 1);
        assert_eq!(session.archivetime_ms, archive.duration_ms());
        session.focus_client = ClientId(99);
        crate::policy::killcam::follow_archived_focus(&archive, &mut session);
        assert_eq!(session.archivetime_ms, 0);
    }

    #[test]
    fn absent_player_state_is_preserved_for_killcam_filtering() {
        let mut archive = FrameArchive::new();
        let mut s = snapshot(1, 0);
        s.players
            .push((ClientId(7), playerstate_iw4::PlayerState::ZERO));
        let frame = ArchivedFrame {
            tick: s.tick,
            snapshot: s,
            no_player_state: vec![ClientId(7)],
        };
        assert!(!frame.player_state_exists(ClientId(7)));
        assert!(archive.push(frame));
        let lookup = archive.lookup(ARCHIVE_TICK_MS);
        let retained = archive.frame(lookup.tick.unwrap()).unwrap();
        assert!(!retained.player_state_exists(ClientId(7)));
        assert_eq!(retained.no_player_state, vec![ClientId(7)]);
        assert_eq!(archive.records(), 1);
    }
}
