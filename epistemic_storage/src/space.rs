use std::collections::HashMap;

use crate::atom::UniversalAtom;
use crate::heat::{EpistemicHeat, PageFault, ThermalMetrics};
use sovereign_core::Cid;

pub trait AtomSpace {
    fn current_heat(&self, cid: &Cid) -> EpistemicHeat;
    fn heat_up(&mut self, cid: &Cid, target: EpistemicHeat) -> Result<(), PageFault>;
    fn cool_down(&mut self, cid: &Cid) -> Result<(), String>;
    fn materialize(&mut self, cid: Cid, atom: UniversalAtom) -> Result<(), String>;
    fn get_thermal_metrics(&self) -> ThermalMetrics;
    fn get_atom(&self, cid: &Cid) -> Option<&UniversalAtom>;
}

#[derive(Debug, Default)]
pub struct InMemoryAtomSpace {
    pub atoms: HashMap<Cid, UniversalAtom>,
    pub heats: HashMap<Cid, EpistemicHeat>,
}

impl AtomSpace for InMemoryAtomSpace {
    fn current_heat(&self, cid: &Cid) -> EpistemicHeat {
        self.heats
            .get(cid)
            .copied()
            .unwrap_or(EpistemicHeat::Absent)
    }

    fn heat_up(&mut self, cid: &Cid, target: EpistemicHeat) -> Result<(), PageFault> {
        if !self.atoms.contains_key(cid) {
            return Err(PageFault::NetworkRequired { cid: cid.clone() });
        }
        self.heats.insert(cid.clone(), target);
        Ok(())
    }

    fn cool_down(&mut self, cid: &Cid) -> Result<(), String> {
        if self.atoms.contains_key(cid) {
            self.heats.insert(cid.clone(), EpistemicHeat::Cold);
            Ok(())
        } else {
            Err("unknown cid".to_string())
        }
    }

    fn materialize(&mut self, cid: Cid, atom: UniversalAtom) -> Result<(), String> {
        self.atoms.insert(cid.clone(), atom);
        self.heats.insert(cid, EpistemicHeat::Cold);
        Ok(())
    }

    fn get_thermal_metrics(&self) -> ThermalMetrics {
        let mut out = ThermalMetrics::default();
        for heat in self.heats.values() {
            match heat {
                EpistemicHeat::Hot => out.hot_atoms += 1,
                EpistemicHeat::Warm => out.warm_atoms += 1,
                EpistemicHeat::Cold => out.cold_atoms += 1,
                EpistemicHeat::Absent => {}
            }
        }
        out
    }

    fn get_atom(&self, cid: &Cid) -> Option<&UniversalAtom> {
        self.atoms.get(cid)
    }
}
