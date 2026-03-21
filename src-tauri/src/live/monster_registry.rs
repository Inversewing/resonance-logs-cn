use anyhow::{Context, Result};
use log::warn;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MonsterType {
    Normal = 0,
    Elite = 1,
    Boss = 2,
}

#[derive(Debug, Clone)]
pub struct MonsterInfo {
    pub name: String,
    pub monster_type: MonsterType,
}

const EXTRA_BUFF_MONITORED_MONSTERS_RELATIVE: &str = "meter-data/ExtraBuffMonitoredMonsters.json";

static MONSTER_REGISTRY: LazyLock<HashMap<i32, MonsterInfo>> = LazyLock::new(|| {
    #[derive(Deserialize)]
    struct RawMonsterInfo {
        #[serde(rename = "Name")]
        name: String,
        #[serde(rename = "MonsterType")]
        monster_type: u8,
    }

    let data = include_str!("../../meter-data/MonsterIdNameType.json");
    let raw: HashMap<String, RawMonsterInfo> =
        serde_json::from_str(data).expect("invalid MonsterIdNameType.json");

    let mut registry = HashMap::with_capacity(raw.len());
    for (key, info) in raw {
        if let Ok(id) = key.parse::<i32>() {
            let monster_type = match info.monster_type {
                1 => MonsterType::Elite,
                2 => MonsterType::Boss,
                _ => MonsterType::Normal,
            };

            registry.insert(
                id,
                MonsterInfo {
                    name: info.name,
                    monster_type,
                },
            );
        }
    }

    registry
});

static EXTRA_BUFF_MONITORED_MONSTER_IDS: LazyLock<HashSet<i32>> = LazyLock::new(|| {
    load_extra_buff_monitored_monster_ids().unwrap_or_else(|err| {
        warn!(
            "[monster-registry] failed to load {}: {}",
            EXTRA_BUFF_MONITORED_MONSTERS_RELATIVE, err
        );
        HashSet::new()
    })
});

#[derive(Debug, Deserialize)]
struct RawExtraBuffMonitoredMonsters {
    #[serde(rename = "monsterIds")]
    monster_ids: Vec<i32>,
}

fn locate_meter_data_file(relative_path: &str) -> Option<PathBuf> {
    let mut path = PathBuf::from(relative_path);
    if path.exists() {
        return Some(path);
    }

    path = PathBuf::from(format!("src-tauri/{}", relative_path));
    if path.exists() {
        return Some(path);
    }

    if let Ok(mut exe_dir) = std::env::current_exe() {
        exe_dir.pop();
        let candidate = exe_dir.join(relative_path);
        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}

fn parse_extra_buff_monitored_monster_ids(
    contents: &str,
) -> Result<HashSet<i32>, serde_json::Error> {
    let raw: RawExtraBuffMonitoredMonsters = serde_json::from_str(contents)?;
    Ok(raw.monster_ids.into_iter().filter(|id| *id > 0).collect())
}

fn load_extra_buff_monitored_monster_ids() -> Result<HashSet<i32>> {
    let path =
        locate_meter_data_file(EXTRA_BUFF_MONITORED_MONSTERS_RELATIVE).with_context(|| {
            format!(
                "{} not found in known locations",
                EXTRA_BUFF_MONITORED_MONSTERS_RELATIVE
            )
        })?;
    let contents =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    parse_extra_buff_monitored_monster_ids(&contents)
        .with_context(|| format!("failed to parse {}", path.display()))
}

pub fn monster_name(id: i32) -> Option<&'static str> {
    MONSTER_REGISTRY.get(&id).map(|info| info.name.as_str())
}

pub fn monster_type(id: i32) -> Option<MonsterType> {
    MONSTER_REGISTRY.get(&id).map(|info| info.monster_type)
}

pub fn is_extra_buff_monitored_monster(id: i32) -> bool {
    EXTRA_BUFF_MONITORED_MONSTER_IDS.contains(&id)
}
