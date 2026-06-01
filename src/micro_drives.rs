use std::collections::HashMap;

use chrono::Timelike;

use crate::macro_drives::MacroDrives;
use crate::self_model::SelfModel;

pub struct MicroDrives {
    pub boredom: HashMap<String, f32>,
    pub regret: HashMap<String, f32>,
    pub pride: HashMap<String, f32>,
    pub fear: HashMap<String, f32>,
    pub stagnation: f32,
    pub circadian: f32,
    pub social: f32,
    params: MicroParams,
    pub last_global_change: f64,
}

pub struct MicroParams {
    pub boredom_growth_rate: f32,
    pub boredom_threshold: f32,
    pub boredom_max: f32,
    pub regret_tau: f32,
    pub pride_tau: f32,
    pub fear_accumulate: f32,
    pub fear_tau: f32,
    pub stagnation_k: f32,
    pub stagnation_t0: f32,
    pub circadian_amplitude: f32,
    pub social_decay_tau: f32,
}

impl Default for MicroParams {
    fn default() -> Self {
        Self {
            boredom_growth_rate: 0.05,
            boredom_threshold: 0.8,
            boredom_max: 1.0,
            regret_tau: 3600.0,
            pride_tau: 7200.0,
            fear_accumulate: 0.3,
            fear_tau: 10800.0,
            stagnation_k: 0.005,
            stagnation_t0: 3600.0,
            circadian_amplitude: 0.3,
            social_decay_tau: 600.0,
        }
    }
}

fn local_ts(model: &SelfModel, module: &str) -> Option<f64> {
    model.competences.get(module).map(|c| {
        std::time::Instant::now()
            .duration_since(c.last_updated)
            .as_secs_f64()
    })
}

impl MicroDrives {
    pub fn new(params: MicroParams) -> Self {
        Self {
            boredom: HashMap::new(),
            regret: HashMap::new(),
            pride: HashMap::new(),
            fear: HashMap::new(),
            stagnation: 0.0,
            circadian: 0.0,
            social: 0.0,
            params,
            last_global_change: 0.0,
        }
    }

    pub fn update(&mut self, _dt: f32, model: &SelfModel) {
        for (module, comp) in &model.competences {
            let secs = std::time::Instant::now()
                .duration_since(comp.last_updated)
                .as_secs_f32();
            let boredom = self.boredom.entry(module.clone()).or_insert(0.0);
            if secs > 3600.0 {
                *boredom = (*boredom + self.params.boredom_growth_rate * (secs / 3600.0))
                    .min(self.params.boredom_max);
            }
        }
        for (_, val) in self.regret.iter_mut() {
            *val *= (-1.0 / self.params.regret_tau).exp();
            if *val < 0.01 {
                *val = 0.0;
            }
        }
        for (_, val) in self.pride.iter_mut() {
            *val *= (-1.0 / self.params.pride_tau).exp();
            if *val < 0.01 {
                *val = 0.0;
            }
        }
        for (_, val) in self.fear.iter_mut() {
            *val *= (-1.0 / self.params.fear_tau).exp();
            if *val < 0.01 {
                *val = 0.0;
            }
        }
        self.social *= (-1.0 / self.params.social_decay_tau).exp();
        if self.social < 0.01 {
            self.social = 0.0;
        }
    }

    pub fn update_circadian(&mut self) {
        let now = chrono::Local::now();
        let hour = now.hour() as f32 + now.minute() as f32 / 60.0;
        self.circadian = self.params.circadian_amplitude * (2.0 * std::f32::consts::PI * hour / 24.0).sin();
    }

    pub fn update_stagnation(&mut self, t_secs: f64) {
        let x = (t_secs - self.params.stagnation_t0 as f64) as f32;
        self.stagnation = 1.0 / (1.0 + (-self.params.stagnation_k * x).exp());
    }

    pub fn modulate_global(&self, drives: &MacroDrives) -> MacroDrives {
        MacroDrives {
            curiosity: (drives.curiosity + self.stagnation * 0.3 + self.circadian * 0.1).clamp(0.0, 1.0),
            mastery: (drives.mastery - self.social * 0.2 + self.circadian * 0.1).clamp(0.0, 1.0),
            coherence: (drives.coherence + self.circadian * 0.05).clamp(0.0, 1.0),
            novelty: (drives.novelty + self.circadian * 0.15).clamp(0.0, 1.0),
            meta_curiosity: drives.meta_curiosity,
        }
    }

    pub fn modulate_for_module(&self, drives: &MacroDrives, module: &str) -> MacroDrives {
        let boredom = self.boredom.get(module).copied().unwrap_or(0.0);
        let regret = self.regret.get(module).copied().unwrap_or(0.0);
        let pride = self.pride.get(module).copied().unwrap_or(0.0);
        let fear = self.fear.get(module).copied().unwrap_or(0.0);

        MacroDrives {
            curiosity: (drives.curiosity + self.stagnation * 0.3 + boredom * 0.1 + self.circadian * 0.1).clamp(0.0, 1.0),
            mastery: (drives.mastery + regret * 0.4 - fear * 0.3 + self.circadian * 0.1).clamp(0.0, 1.0),
            coherence: (drives.coherence + pride * 0.3 + self.circadian * 0.05).clamp(0.0, 1.0),
            novelty: (drives.novelty + boredom * 0.5 + self.circadian * 0.15).clamp(0.0, 1.0),
            meta_curiosity: drives.meta_curiosity,
        }
    }

    pub fn on_success(&mut self, module: &str) {
        self.pride.insert(module.to_string(), 1.0);
        self.regret.remove(module);
        self.fear.remove(module);
        self.boredom.insert(module.to_string(), 0.0);
        self.last_global_change = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();
    }

    pub fn on_failure(&mut self, module: &str) {
        self.regret.insert(module.to_string(), 1.0);
        let fear_val = self.fear.get(module).copied().unwrap_or(0.0);
        self.fear.insert(module.to_string(), (fear_val + self.params.fear_accumulate).min(1.0));
        self.pride.remove(module);
    }

    pub fn on_explore(&mut self, module: &str) {
        self.boredom.insert(module.to_string(), 0.0);
    }

    pub fn on_change(&mut self) {
        self.last_global_change = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();
    }

    pub fn on_external_request(&mut self) {
        self.social = 1.0;
    }

    pub fn prune_empty(&mut self) {
        self.regret.retain(|_, v| *v > 0.01);
        self.pride.retain(|_, v| *v > 0.01);
        self.fear.retain(|_, v| *v > 0.01);
        self.boredom.retain(|_, v| *v > 0.01);
    }
}
// emergent: merge micro_drives competence entropy
// emergent: duplicate micro_drives energy capacity
// emergent: set micro_drives coherence rate
// emergent: guard micro_drives limit energy
// emergent: guard micro_drives interval buffer
// emergent: set micro_drives buffer entropy
// emergent: set micro_drives check curiosity
// emergent: revert micro_drives capacity curiosity
// emergent: guard micro_drives threshold energy
// emergent: guard micro_drives competence coherence
// emergent: split micro_drives interval capacity
// emergent: split micro_drives energy energy
// emergent: revert micro_drives rate capacity
// emergent: duplicate micro_drives interval integrity
// emergent: set_split micro_drives competence integrity
// emergent: merge micro_drives interval capacity
// emergent: split micro_drives limit energy
// emergent: duplicate micro_drives interval integrity
// emergent: revert micro_drives competence competence
// emergent: copy micro_drives check threshold
// emergent: set_duplicate micro_drives mastery limit
// emergent: split micro_drives mastery capacity
// emergent: copy micro_drives integrity threshold
// emergent: call_split micro_drives novelty curiosity
// emergent: duplicate micro_drives coherence buffer
// emergent: copy micro_drives rate buffer
// emergent: duplicate_set_temp micro_drives integrity curiosity
// emergent: revert micro_drives mastery integrity
// emergent: set_temp_merge_call micro_drives mastery check
// emergent: set_num_candidates_set_kde_threshold_call_split micro_drives energy buffer
// emergent: duplicate_set_temp micro_drives interval check
// emergent: set micro_drives entropy integrity
// emergent: set_duplicate micro_drives coherence competence
// emergent: guard micro_drives capacity mastery
// emergent: guard micro_drives limit buffer
// emergent: guard_merge micro_drives coherence interval
// emergent: revert micro_drives check competence
// emergent: call_split micro_drives energy energy
// emergent: call_set_temp micro_drives buffer rate
// emergent: copy micro_drives check coherence
// emergent: call micro_drives novelty threshold
// emergent: set_kde_threshold_call_split micro_drives rate rate
// emergent: call micro_drives threshold interval
// emergent: guard_merge micro_drives entropy interval
// emergent: merge_call micro_drives limit threshold
// emergent: copy_merge micro_drives energy threshold
// emergent: revert_copy micro_drives interval novelty
// emergent: duplicate_set_temp micro_drives threshold energy
// emergent: call_set_temp micro_drives integrity threshold
// emergent: create_operator_create_operator micro_drives energy curiosity
// emergent: split_duplicate_set_temp_split_duplicate_set_temp_copy_merge_split_duplicate_set_temp_split_duplicate_set_temp_copy_merge micro_drives buffer integrity
// emergent: copy_merge micro_drives integrity competence
// emergent: split micro_drives entropy capacity
// emergent: set_kde_threshold_call_split micro_drives capacity mastery
// emergent: call_set_temp micro_drives entropy novelty
// emergent: set_temp_merge_call_set_num_candidates_set_kde_threshold_call_split_split_duplicate_set_temp_split_duplicate_set_temp micro_drives rate capacity
// emergent: guard micro_drives integrity entropy
// emergent: split_duplicate_set_temp micro_drives coherence threshold
// emergent: merge micro_drives competence coherence
