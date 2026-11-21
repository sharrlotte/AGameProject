use std::collections::{HashMap, HashSet};

use crate::quest::component::{
    QuestAcceptance, QuestDef, QuestId, QuestObjectiveDef,
};
use crate::validate::PositiveF32;

pub fn validate_catalog(quests: &[QuestDef]) -> Result<(), String> {
    let mut ids = HashSet::new();
    for quest in quests {
        if let QuestId::Authored(id) = &quest.id
            && id.trim().is_empty()
        {
            return Err("quest id must not be empty".to_string());
        }
        if !ids.insert(quest.id.clone()) {
            return Err(format!("duplicate quest id {}", quest.id.key()));
        }
        if quest.title.trim().is_empty() {
            return Err(format!("quest {} has an empty title", quest.id.key()));
        }
        if quest.objectives.is_empty() {
            return Err(format!("quest {} has no objectives", quest.id.key()));
        }

        let mut objective_ids = HashSet::new();
        for objective in &quest.objectives {
            if objective.id().trim().is_empty() {
                return Err(format!("quest {} has an objective with an empty id", quest.id.key()));
            }
            if !objective_ids.insert(objective.id()) {
                return Err(format!("quest {} has duplicate objective id {}", quest.id.key(), objective.id()));
            }
            match objective {
                QuestObjectiveDef::DestroyEnemies { count, .. }
                | QuestObjectiveDef::CollectItem { count, .. } => {
                    if *count < 1 { Err(format!("must be at least 1, got {}", *count)) } else { Ok(()) }
                        .map_err(|e| format!("quest {}: {e}", quest.id.key()))?;
                }
                QuestObjectiveDef::ReachPosition { radius, .. } => {
                    PositiveF32::new(*radius).map(|_| ())
                        .map_err(|e| format!("quest {}: {e}", quest.id.key()))?;
                }
            }
        }

        if quest.acceptance == QuestAcceptance::Manual {
            return Err(format!("authored quest {} cannot be Manual", quest.id.key()));
        }

        if let Some(offer) = &quest.offer {
            PositiveF32::new(offer.radius).map(|_| ())
                .map_err(|e| format!("quest {}: {e}", quest.id.key()))?;
        }
    }

    for quest in quests {
        for required in &quest.requires {
            if !ids.contains(required) {
                return Err(format!(
                    "quest {} requires unknown quest {}",
                    quest.id.key(),
                    required.key()
                ));
            }
        }
    }

    let graph: HashMap<&QuestId, &Vec<QuestId>> = quests
        .iter()
        .map(|quest| (&quest.id, &quest.requires))
        .collect();

    let mut visiting = HashSet::new();
    let mut visited = HashSet::new();
    
    for quest in quests {
        check_acyclic(&quest.id, &graph, &mut visiting, &mut visited)?;
    }

    Ok(())
}

fn check_acyclic(
    id: &QuestId,
    graph: &HashMap<&QuestId, &Vec<QuestId>>,
    visiting: &mut HashSet<QuestId>,
    visited: &mut HashSet<QuestId>,
) -> Result<(), String> {
    if visited.contains(id) {
        return Ok(());
    }
    if !visiting.insert(id.clone()) {
        return Err(format!("prerequisite cycle detected at {}", id.key()));
    }
    if let Some(requires) = graph.get(id) {
        for required in requires.iter() {
            check_acyclic(required, graph, visiting, visited)?;
        }
    }
    visiting.remove(id);
    visited.insert(id.clone());
    Ok(())
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::*;
    use crate::quest::component::{QuestDef, QuestOffer, QuestPrereqMode};

    fn quest(id: &str, requires: Vec<QuestId>) -> QuestDef {
        QuestDef {
            id: QuestId::Authored(id.to_string()),
            title: id.to_string(),
            description: String::new(),
            requires,
            requires_mode: QuestPrereqMode::default(),
            acceptance: QuestAcceptance::default(),
            offer: None,
            objectives: vec![QuestObjectiveDef::DestroyEnemies {
                id: "kills".to_string(),
                count: 1,
            }],
        }
    }

    #[test]
    fn cycle_is_rejected() {
        let quests = vec![
            quest("a", vec![QuestId::Authored("b".to_string())]),
            quest("b", vec![QuestId::Authored("a".to_string())]),
        ];
        let err = validate_catalog(&quests).expect_err("cycle must fail");
        assert!(err.contains("cycle"), "{err}");
    }

    #[test]
    fn unknown_prerequisite_is_rejected() {
        let quests = vec![quest("a", vec![QuestId::Authored("missing".to_string())])];
        let err = validate_catalog(&quests).expect_err("unknown requires must fail");
        assert!(err.contains("unknown quest"), "{err}");
    }

    #[test]
    fn authored_manual_quest_is_rejected() {
        let mut manual = quest("a", Vec::new());
        manual.acceptance = QuestAcceptance::Manual;
        manual.offer = Some(QuestOffer {
            position: Vec2::ZERO,
            radius: 1.0,
        });
        let err = validate_catalog(&[manual]).expect_err("manual authored must fail");
        assert!(err.contains("cannot be Manual"), "{err}");
    }
}
