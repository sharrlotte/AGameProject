use bevy::prelude::*;

use crate::quest::component::{
    QuestAcceptance, QuestDef, QuestGenConfig, QuestId, QuestObjectiveDef, QuestOffer,
};
use crate::resource::GameRng;

pub fn build(chunk: IVec2, config: &QuestGenConfig, rng: &mut GameRng) -> QuestDef {
    let center = chunk.as_vec2() * config.chunk_size
        + Vec2::splat(config.chunk_size * 0.5);

    let acceptance = if (chunk.x.unsigned_abs() + chunk.y.unsigned_abs())
        % config.manual_acceptance_every.max(1)
        == 0
    {
        QuestAcceptance::Manual
    } else {
        QuestAcceptance::Auto
    };

    let objective_count = 1 + (rng.next_u64() % 2) as usize;
    let mut objectives = Vec::with_capacity(objective_count);
    for index in 0..objective_count {
        let id = format!("obj_{index}");
        match rng.next_u64() % 3 {
            0 => objectives.push(QuestObjectiveDef::DestroyEnemies {
                id,
                count: range_u32(rng, config.destroy_count),
            }),
            1 => objectives.push(QuestObjectiveDef::CollectItem {
                id,
                item: pick_item(rng, &config.item_pool),
                count: range_u32(rng, config.collect_count),
            }),
            _ => objectives.push(QuestObjectiveDef::ReachPosition {
                id,
                position: center + Vec2::new(range_f32(rng, -2000.0, 2000.0), range_f32(rng, -2000.0, 2000.0)),
                radius: range_f32(rng, config.reach_radius.0, config.reach_radius.1),
            }),
        }
    }

    let title = match acceptance {
        QuestAcceptance::Auto => format!("Frontier Contract {}:{}", chunk.x, chunk.y),
        QuestAcceptance::Manual => format!("Local Request {}:{}", chunk.x, chunk.y),
    };

    QuestDef {
        id: QuestId::Chunk(chunk),
        title,
        description: format!("Sector {}:{}. Survey and secure the region.", chunk.x, chunk.y),
        requires: Vec::new(),
        requires_mode: Default::default(),
        acceptance,
        offer: Some(QuestOffer {
            position: center,
            radius: config.notification_radius,
        }),
        objectives,
    }
}

pub fn validate_generated(def: &QuestDef) -> Result<(), String> {
    if !matches!(def.id, QuestId::Chunk(_)) {
        return Err("generated quest must have a chunk id".to_string());
    }
    if def.objectives.is_empty() {
        return Err("generated quest must have objectives".to_string());
    }
    if def.acceptance == QuestAcceptance::Manual && def.offer.is_none() {
        return Err(format!(
            "manual generated quest {} has no offer",
            def.id.key()
        ));
    }
    Ok(())
}

fn range_u32(rng: &mut GameRng, range: (u32, u32)) -> u32 {
    let (min, max) = if range.0 <= range.1 {
        range
    } else {
        (range.1, range.0)
    };
    min + (rng.next_u64() % (u64::from(max - min) + 1)) as u32
}

fn range_f32(rng: &mut GameRng, min: f32, max: f32) -> f32 {
    let span = (max - min).max(0.0);
    let ratio = (rng.next_u64() % 10_000) as f32 / 10_000.0;
    min + span * ratio
}

fn pick_item(rng: &mut GameRng, pool: &[String]) -> String {
    if pool.is_empty() {
        return "steel".to_string();
    }
    let index = (rng.next_u64() % pool.len() as u64) as usize;
    pool[index].clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> QuestGenConfig {
        QuestGenConfig::default()
    }

    #[test]
    fn generated_id_is_chunk_derived_and_deterministic() {
        let mut a = GameRng::new(42);
        let mut b = GameRng::new(42);
        let first = build(IVec2::new(2, -3), &config(), &mut a);
        let second = build(IVec2::new(2, -3), &config(), &mut b);
        assert_eq!(first.id, QuestId::Chunk(IVec2::new(2, -3)));
        assert_eq!(first, second);
    }

    #[test]
    fn generated_quest_has_offer_and_valid_objectives() {
        let def = build(IVec2::new(1, 0), &config(), &mut GameRng::new(7));
        validate_generated(&def).expect("generated quest valid");
        assert!(def.offer.is_some());
        assert!(!def.objectives.is_empty());
    }

    #[test]
    fn manual_quest_without_offer_is_rejected() {
        let mut def = build(IVec2::new(0, 0), &config(), &mut GameRng::new(3));
        def.acceptance = QuestAcceptance::Manual;
        def.offer = None;
        let error = validate_generated(&def).expect_err("missing offer must fail");
        assert!(error.contains("no offer"), "{error}");
    }
}
