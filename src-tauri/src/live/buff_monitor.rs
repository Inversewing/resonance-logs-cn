use crate::database::now_ms;
use crate::live::commands_models::BuffUpdateState;
use blueprotobuf_lib::blueprotobuf::{
    BuffChange, BuffEffectSync, BuffInfo, EBuffEffectLogicPbType, EBuffEventType,
};
use prost::Message;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ActiveBuff {
    pub buff_uuid: i32,
    pub base_id: i32,
    pub layer: i32,
    pub duration: i32,
    pub create_time: i64,
    pub source_config_id: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuffChangeType {
    Added,
    Changed,
    Removed,
}

#[derive(Debug, Clone)]
pub struct BuffChangeEvent {
    pub base_id: i32,
    pub change_type: BuffChangeType,
}

#[derive(Debug, Default)]
pub struct BuffProcessResult {
    pub update_payload: Option<Vec<BuffUpdateState>>,
    pub changes: Vec<BuffChangeEvent>,
}

#[derive(Debug, Default)]
pub struct BuffMonitor {
    /// Ordered list of monitored buff base IDs.
    pub monitored_buff_ids: Vec<i32>,
    /// Active buffs keyed by buff UUID.
    pub active_buffs: HashMap<i32, ActiveBuff>,
    /// Monitor all buffs.
    pub monitor_all_buff: bool,
}

impl BuffMonitor {
    pub(crate) fn new() -> Self {
        Self {
            monitored_buff_ids: Vec::new(),
            active_buffs: HashMap::new(),
            monitor_all_buff: false,
        }
    }

    pub(crate) fn process_buff_effect_bytes(
        &mut self,
        raw_bytes: &[u8],
        server_clock_offset: &mut i64,
    ) -> BuffProcessResult {
        let mut changes = Vec::new();
        let Ok(buff_effect_sync) = BuffEffectSync::decode(raw_bytes) else {
            return BuffProcessResult::default();
        };
        let now = now_ms();

        for buff_effect in buff_effect_sync.buff_effects {
            let buff_uuid = match buff_effect.buff_uuid {
                Some(id) => id,
                None => continue,
            };

            for logic_effect in buff_effect.logic_effect {
                let Some(effect_type) = logic_effect.effect_type else {
                    continue;
                };
                let Some(raw) = logic_effect.raw_data else {
                    continue;
                };

                if effect_type == EBuffEffectLogicPbType::BuffEffectAddBuff as i32 {
                    if let Ok(buff_info) = BuffInfo::decode(raw.as_slice()) {
                        let Some(base_id) = buff_info.base_id else {
                            continue;
                        };
                        let layer = buff_info.layer.unwrap_or(1);
                        let duration = buff_info.duration.unwrap_or(0);
                        let create_time = buff_info.create_time.unwrap_or(now);
                        if buff_info.create_time.is_some() {
                            *server_clock_offset = now - create_time;
                        }
                        let source_config_id = buff_info
                            .fight_source_info
                            .and_then(|info| info.source_config_id)
                            .unwrap_or(0);

                        self.active_buffs.insert(
                            buff_uuid,
                            ActiveBuff {
                                buff_uuid,
                                base_id,
                                layer,
                                duration,
                                create_time,
                                source_config_id,
                            },
                        );
                        changes.push(BuffChangeEvent {
                            base_id,
                            change_type: BuffChangeType::Added,
                        });
                    }
                } else if effect_type == EBuffEffectLogicPbType::BuffEffectBuffChange as i32 {
                    if let Ok(change_info) = BuffChange::decode(raw.as_slice()) {
                        if let Some(entry) = self.active_buffs.get_mut(&buff_uuid) {
                            let base_id = entry.base_id;
                            if let Some(layer) = change_info.layer {
                                entry.layer = layer;
                            }
                            if let Some(duration) = change_info.duration {
                                entry.duration = duration;
                            }
                            if let Some(create_time) = change_info.create_time {
                                entry.create_time = create_time;
                            }
                            changes.push(BuffChangeEvent {
                                base_id,
                                change_type: BuffChangeType::Changed,
                            });
                        }
                    }
                }
            }

            if buff_effect.r#type == Some(EBuffEventType::BuffEventRemove as i32) {
                let removed_buff = self.active_buffs.remove(&buff_uuid);
                if let Some(removed_buff) = removed_buff {
                    changes.push(BuffChangeEvent {
                        base_id: removed_buff.base_id,
                        change_type: BuffChangeType::Removed,
                    });
                }
            }
        }

        let update_payload = if self.monitored_buff_ids.is_empty() && !self.monitor_all_buff {
            None
        } else {
            Some(
                self.active_buffs
                    .values()
                    .filter(|buff| {
                        self.monitor_all_buff || self.monitored_buff_ids.contains(&buff.base_id)
                    })
                    .map(|buff| BuffUpdateState {
                        buff_uuid: buff.buff_uuid,
                        base_id: buff.base_id,
                        layer: buff.layer,
                        duration_ms: buff.duration,
                        create_time_ms: buff.create_time.saturating_add(*server_clock_offset),
                        source_config_id: buff.source_config_id,
                    })
                    .collect(),
            )
        };
        BuffProcessResult {
            update_payload,
            changes,
        }
    }
}
