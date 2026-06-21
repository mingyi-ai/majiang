/// MCR fan types.  Each variant maps to exactly one MCR scoring pattern.
///
/// Enum discriminants serve as stable identifiers for bitset exclusion masks.
/// The full 81 MCR fans will be added incrementally; only the ones needed
/// for initial development are defined here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum FanType {
    // ── 88-point fans ──
    BigFourWinds = 1,
    // ── 64-point fans ──
    LittleFourWinds = 2,
    AllHonors = 3,
    // ── 32-point fans ──
    AllTerminalsAndHonors = 4,
    // ── 6-point fans ──
    HalfFlush = 5,
    AllPungs = 6,
}

impl FanType {
    /// Base point value for this fan type (MCR standard).
    pub fn points(self) -> u8 {
        match self {
            Self::BigFourWinds => 88,
            Self::LittleFourWinds | Self::AllHonors => 64,
            Self::AllTerminalsAndHonors => 32,
            Self::HalfFlush | Self::AllPungs => 6,
        }
    }

    /// Human-readable display name.
    pub fn name(self) -> &'static str {
        match self {
            Self::BigFourWinds => "Big Four Winds",
            Self::LittleFourWinds => "Little Four Winds",
            Self::AllHonors => "All Honors",
            Self::AllTerminalsAndHonors => "All Terminals and Honors",
            Self::HalfFlush => "Half Flush",
            Self::AllPungs => "All Pungs",
        }
    }

    /// Bit index within a `FanExclusionSet` this fan occupies.
    pub fn bit_index(self) -> usize {
        self as u16 as usize
    }
}

/// A compact bitset for fan-exclusion checks.
///
/// Each bit corresponds to a `FanType` variant by its discriminant.
/// MCR has at most 81 fans, so `u128` (128 bits) is sufficient.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FanExclusionSet(pub u128);

impl FanExclusionSet {
    pub fn is_set(self, fan: FanType) -> bool {
        (self.0 >> fan.bit_index()) & 1 == 1
    }

    pub fn set(&mut self, fan: FanType) {
        self.0 |= 1u128 << fan.bit_index();
    }

    /// Build the exclusion mask for a fan type: which fans it excludes.
    pub fn for_fan(fan: FanType) -> Self {
        match fan {
            FanType::BigFourWinds => {
                // Big Four Winds implies All Pungs; mutually exclusive with
                // Little Four Winds.
                let mut s = Self::default();
                s.set(FanType::AllPungs);
                s.set(FanType::LittleFourWinds);
                s
            }
            FanType::LittleFourWinds => {
                let mut s = Self::default();
                s.set(FanType::BigFourWinds);
                s
            }
            FanType::AllHonors => {
                // All Honors is strictly implied by All Terminals and Honors,
                // so they can't coexist.
                let mut s = Self::default();
                s.set(FanType::AllTerminalsAndHonors);
                s
            }
            FanType::AllTerminalsAndHonors => {
                // Implies All Pungs (if all pungs). Actually not always —
                // could be a mix of chows and pungs with terminals/honors.
                // For now, no exclusion beyond All Honors.
                let mut s = Self::default();
                s.set(FanType::AllHonors);
                s
            }
            FanType::HalfFlush => FanExclusionSet::default(),
            FanType::AllPungs => FanExclusionSet::default(),
        }
    }
}

// ── Search types ──

/// A candidate fan instance extracted from a decomposition.
/// The search algorithm selects a compatible subset of these.
#[derive(Debug, Clone)]
pub struct FanCandidate {
    pub fan_type: FanType,
    /// Bitmask identifying which of the 4 sets this fan uses.
    /// Bit 0 = sets[0], bit 1 = sets[1], etc.
    pub used_set_mask: u64,
    /// Whether this fan's condition involves the pair.
    pub uses_pair: bool,
    pub score: u8,
    pub excludes_mask: FanExclusionSet,
}

/// A fan instance selected in the final result.
#[derive(Debug, Clone)]
pub struct FanInstance {
    pub fan_type: FanType,
    pub used_set_mask: u64,
    pub uses_pair: bool,
    pub score: u8,
}

/// Result of a fan search.
#[derive(Debug, Clone, Default)]
pub struct FanSolveResult {
    pub total_score: u16,
    pub fans: Vec<FanInstance>,
}
