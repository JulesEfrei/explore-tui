use std::{
    collections::VecDeque,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread::{self, JoinHandle},
};

use crate::{
    bots::{
        BotEvent, BotKind, BotSnapshot, BotStatus, ExplorationBias, MinerAlgorithm, ScoutAlgorithm,
        ScoutMemory, miner_path,
        movement::{MovementProgress, MovementStep, wait_for_tick},
        scout_path,
    },
    map::{Map, MineralKind, Point},
};

pub(super) struct ScoutSpawn {
    pub id: u32,
    pub spawn: Point,
    pub base: Point,
    pub map: Arc<Map>,
    pub events_tx: mpsc::Sender<BotEvent>,
    pub tick_rx: mpsc::Receiver<()>,
    pub stop_flag: Arc<AtomicBool>,
    pub algorithm: ScoutAlgorithm,
    pub bias: ExplorationBias,
}

pub(super) fn spawn_scout(config: ScoutSpawn) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut pos = config.spawn;
        let mut memory = ScoutMemory::new(pos);
        let mut path = VecDeque::new();
        let mut movement = MovementProgress::default();
        let mut pending_discovery: Option<(Point, MineralKind)> = None;

        send_move(&config.events_tx, config.id, pos, BotStatus::Exploring);

        while !config.stop_flag.load(Ordering::Relaxed) {
            if !wait_for_tick(&config.tick_rx, &config.stop_flag) {
                break;
            }

            if path.is_empty() {
                let next_path = if pending_discovery.is_some() {
                    miner_path(&config.map, pos, config.base, MinerAlgorithm::AStar)
                } else {
                    scout_path(
                        &config.map,
                        &memory,
                        pos,
                        config.base,
                        config.algorithm,
                        config.bias,
                    )
                };

                path = next_path.unwrap_or_default().into();
            }

            match movement.advance(&config.map, &mut path) {
                MovementStep::Arrived(next) => {
                    pos = next;
                    memory.mark_visited(pos);
                    report_scout_step(
                        &config.map,
                        &config.events_tx,
                        config.id,
                        pos,
                        config.base,
                        &mut path,
                        &mut pending_discovery,
                    );
                }
                MovementStep::Waiting | MovementStep::Idle => {}
            }
        }
    })
}

fn report_scout_step(
    map: &Map,
    tx: &mpsc::Sender<BotEvent>,
    id: u32,
    pos: Point,
    base: Point,
    path: &mut VecDeque<Point>,
    pending_discovery: &mut Option<(Point, MineralKind)>,
) {
    if let Some((mineral_pos, kind)) = *pending_discovery {
        send_move(tx, id, pos, BotStatus::ReturningToBase);
        if pos == base {
            let _ = tx.send(BotEvent::MineralFound {
                scout_id: id,
                pos: mineral_pos,
                kind,
            });
            *pending_discovery = None;
        }
        return;
    }

    send_move(tx, id, pos, BotStatus::Exploring);
    if let Some(mineral) = map.mineral_at(pos) {
        *pending_discovery = Some((pos, mineral.kind));
        path.clear();
    }
}

fn send_move(tx: &mpsc::Sender<BotEvent>, id: u32, pos: Point, status: BotStatus) {
    let _ = tx.send(BotEvent::BotMoved(BotSnapshot {
        id,
        kind: BotKind::Scout,
        pos,
        status,
    }));
}
