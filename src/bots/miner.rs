use std::{
    collections::VecDeque,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, TryRecvError},
    },
    thread::{self, JoinHandle},
};

use crate::{
    bots::{
        BotEvent, BotKind, BotSnapshot, BotStatus, MinerAlgorithm, MiningOrder, miner_path,
        movement::{MovementProgress, wait_for_tick},
    },
    map::{Map, Point},
};

pub(super) struct MinerSpawn {
    pub id: u32,
    pub base: Point,
    pub map: Arc<Map>,
    pub events_tx: mpsc::Sender<BotEvent>,
    pub orders_rx: mpsc::Receiver<MiningOrder>,
    pub tick_rx: mpsc::Receiver<()>,
    pub stop_flag: Arc<AtomicBool>,
    pub algorithm: MinerAlgorithm,
}

pub(super) fn spawn_miner(config: MinerSpawn) -> JoinHandle<()> {
    thread::spawn(move || MinerWorker::new(config).run())
}

struct MinerWorker {
    id: u32,
    base: Point,
    map: Arc<Map>,
    events_tx: mpsc::Sender<BotEvent>,
    orders_rx: mpsc::Receiver<MiningOrder>,
    tick_rx: mpsc::Receiver<()>,
    stop_flag: Arc<AtomicBool>,
    algorithm: MinerAlgorithm,
    pos: Point,
    active_order: Option<MiningOrder>,
    returning_to_base: bool,
    path: VecDeque<Point>,
    movement: MovementProgress,
}

impl MinerWorker {
    fn new(config: MinerSpawn) -> Self {
        Self {
            id: config.id,
            base: config.base,
            map: config.map,
            events_tx: config.events_tx,
            orders_rx: config.orders_rx,
            tick_rx: config.tick_rx,
            stop_flag: config.stop_flag,
            algorithm: config.algorithm,
            pos: config.base,
            active_order: None,
            returning_to_base: false,
            path: VecDeque::new(),
            movement: MovementProgress::default(),
        }
    }

    fn run(&mut self) {
        send_miner_move(&self.events_tx, self.id, self.pos, BotStatus::Idle);

        while !self.stop_flag.load(Ordering::Relaxed) {
            if !wait_for_tick(&self.tick_rx, &self.stop_flag) {
                break;
            }

            if self.active_order.is_none() && !self.try_start_order() {
                send_miner_move(&self.events_tx, self.id, self.pos, BotStatus::Idle);
                continue;
            }

            if let Some(next) = self.movement.advance(&self.map, &mut self.path) {
                self.pos = next;
                let status = if self.returning_to_base {
                    BotStatus::ReturningToBase
                } else {
                    BotStatus::Moving
                };
                send_miner_move(&self.events_tx, self.id, self.pos, status);
                continue;
            }

            self.complete_phase();
        }
    }

    fn try_start_order(&mut self) -> bool {
        match self.orders_rx.try_recv() {
            Ok(order) if order.miner_id == self.id => {
                self.path = miner_path(&self.map, self.pos, order.pos, self.algorithm)
                    .unwrap_or_default()
                    .into();
                self.active_order = Some(order);
                self.returning_to_base = false;
                true
            }
            Ok(_) | Err(TryRecvError::Empty) => false,
            Err(TryRecvError::Disconnected) => false,
        }
    }

    fn complete_phase(&mut self) {
        let Some(order) = self.active_order else {
            return;
        };

        if !self.returning_to_base {
            let _ = self.events_tx.send(BotEvent::MinerArrivedAtMineral {
                miner_id: self.id,
                pos: order.pos,
            });
            send_miner_move(&self.events_tx, self.id, self.pos, BotStatus::Mining);
            self.path = miner_path(&self.map, self.pos, self.base, self.algorithm)
                .unwrap_or_default()
                .into();
            self.returning_to_base = true;
        } else {
            let _ = self.events_tx.send(BotEvent::ResourcesDelivered {
                miner_id: self.id,
                pos: order.pos,
                amount: 1,
            });
            self.active_order = None;
            self.returning_to_base = false;
            send_miner_move(&self.events_tx, self.id, self.pos, BotStatus::Idle);
        }
    }
}

fn send_miner_move(tx: &mpsc::Sender<BotEvent>, id: u32, pos: Point, status: BotStatus) {
    let _ = tx.send(BotEvent::BotMoved(BotSnapshot {
        id,
        kind: BotKind::Miner,
        pos,
        status,
    }));
}
