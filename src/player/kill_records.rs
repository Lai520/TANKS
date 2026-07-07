pub const ENEMY_TANK_TYPE_COUNT: usize = 3;

/// 某一类敌坦克的击毁记录
#[derive(Clone, Copy, Debug, Default)]
pub struct EnemyTypeKill {
    pub count: u32,
    pub points_per_kill: usize,
}

/// 玩家击毁敌坦克记录（按类型 1~3）
#[derive(Clone, Copy, Debug, Default)]
pub struct EnemyKillRecords {
    pub types: [EnemyTypeKill; ENEMY_TANK_TYPE_COUNT],
}

impl EnemyKillRecords {
    pub fn record(&mut self, tank_type: u8, score: usize) {
        let Some(slot) = self.types.get_mut(tank_type as usize - 1) else {
            return;
        };
        slot.count += 1;
        slot.points_per_kill = score;
    }

    pub fn count_for_type(&self, type_index: usize) -> u32 {
        self.types[type_index].count
    }

    pub fn points_for_type(&self, type_index: usize) -> usize {
        let entry = &self.types[type_index];
        entry.count as usize * entry.points_per_kill
    }

    pub fn points_per_kill(&self, type_index: usize) -> usize {
        self.types[type_index].points_per_kill
    }

    pub fn total_kills(&self) -> u32 {
        self.types.iter().map(|entry| entry.count).sum()
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}
