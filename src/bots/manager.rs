use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread::JoinHandle,
};

use crate::{
    bots::{
        AssignmentStrategy, BotEvent, ExplorationBias, MinerAlgorithm, MiningOrder, ScoutAlgorithm,
        miner::{MinerSpawn, spawn_miner},
        scout::{ScoutSpawn, spawn_scout},
    },
    map::Map,
};

#[derive(Debug, Clone, Copy)]
pub struct BotConfig {
    pub scout_count: u32,
    pub miner_count: u32,
    pub scout_algorithm: ScoutAlgorithm,
    pub miner_algorithm: MinerAlgorithm,
    pub assignment_strategy: AssignmentStrategy,
}

impl Default for BotConfig {
    fn default() -> Self {
        Self {
            scout_count: 3,
            miner_count: 2,
            scout_algorithm: ScoutAlgorithm::FrontierWavefront,
            miner_algorithm: MinerAlgorithm::AStar,
            assignment_strategy: AssignmentStrategy::LeastAssigned,
        }
    }
}

#[derive(Debug)]
pub struct BotManager {
    scouts: Vec<JoinHandle<()>>,
    miners: Vec<JoinHandle<()>>,
    miner_order_txs: HashMap<u32, mpsc::Sender<MiningOrder>>,
    tick_txs: Vec<mpsc::Sender<()>>,
    stop_flag: Arc<AtomicBool>,
}

impl BotManager {
    pub fn spawn(map: Arc<Map>, from_bots_tx: mpsc::Sender<BotEvent>, config: BotConfig) -> Self {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let mut scouts = Vec::with_capacity(config.scout_count as usize);
        let mut miners = Vec::with_capacity(config.miner_count as usize);
        let mut miner_order_txs = HashMap::new();
        let mut tick_txs = Vec::with_capacity((config.scout_count + config.miner_count) as usize);

        if let Some(base) = map.base_center() {
            let spawn_tiles = map.base_tiles().unwrap_or([base; 4]);

            for id in 0..config.scout_count {
                let spawn = spawn_tiles[rand::random_range(0..spawn_tiles.len())];
                let (tick_tx, tick_rx) = mpsc::channel();
                tick_txs.push(tick_tx);
                scouts.push(spawn_scout(ScoutSpawn {
                    id,
                    spawn,
                    base,
                    map: Arc::clone(&map),
                    events_tx: from_bots_tx.clone(),
                    tick_rx,
                    stop_flag: Arc::clone(&stop_flag),
                    algorithm: config.scout_algorithm,
                    bias: ExplorationBias::for_scout(id),
                }));
            }

            for index in 0..config.miner_count {
                let id = config.scout_count + index;
                let (orders_tx, orders_rx) = mpsc::channel();
                let (tick_tx, tick_rx) = mpsc::channel();
                miner_order_txs.insert(id, orders_tx);
                tick_txs.push(tick_tx);
                miners.push(spawn_miner(MinerSpawn {
                    id,
                    base,
                    map: Arc::clone(&map),
                    events_tx: from_bots_tx.clone(),
                    orders_rx,
                    tick_rx,
                    stop_flag: Arc::clone(&stop_flag),
                    algorithm: config.miner_algorithm,
                }));
            }
        }

        Self {
            scouts,
            miners,
            miner_order_txs,
            tick_txs,
            stop_flag,
        }
    }

    pub fn miner_ids(&self) -> Vec<u32> {
        self.miner_order_txs.keys().copied().collect()
    }

    pub fn send_order(&self, order: MiningOrder) -> bool {
        self.miner_order_txs
            .get(&order.miner_id)
            .is_some_and(|tx| tx.send(order).is_ok())
    }

    pub fn tick(&self) {
        for tick_tx in &self.tick_txs {
            let _ = tick_tx.send(());
        }
    }

    pub fn shutdown(&mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        self.tick_txs.clear();
        for scout in self.scouts.drain(..) {
            let _ = scout.join();
        }
        for miner in self.miners.drain(..) {
            let _ = miner.join();
        }
    }
}

impl Drop for BotManager {
    fn drop(&mut self) {
        self.shutdown();
    }
}
