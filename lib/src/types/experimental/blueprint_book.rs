use crate::types::Blueprint;

/// Models a collection of blueprints.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BlueprintBook {
    entries: Vec<Blueprint>,
}

impl BlueprintBook {
    pub const fn from(entries: Vec<Blueprint>) -> Self {
        Self { entries }
    }

    pub fn entries(&self) -> &[Blueprint] {
        &self.entries
    }
}

impl std::fmt::Display for BlueprintBook {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for bp in &self.entries {
            bp.fmt(f)?;
            f.write_str("\n")?;
        }
        Ok(())
    }
}
