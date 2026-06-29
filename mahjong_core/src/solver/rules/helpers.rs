// ═══════════════════════════════════════════════════════════════
// Shared helpers for MCR rule checking
// ═══════════════════════════════════════════════════════════════

use super::{FanCandidate, FanType};
use super::profile::{HandProfile, MeldInfo, MeldKind};

/// Create a FanCandidate with score set from FanType points.
#[inline]
pub(crate) fn cand(ft: FanType, mask: u64, uses_pair: bool) -> FanCandidate {
    FanCandidate {
        fan_type: ft,
        used_set_mask: mask,
        uses_pair,
        score: ft.points(),
        excludes_mask: Default::default(),
    }
}

/// Check if a meld is an exposed (melded) kong.
#[inline]
pub(crate) fn is_melded_kong(m: &MeldInfo) -> bool {
    matches!(m.kind, MeldKind::Kong) && !m.is_concealed
}

/// Check if a meld is pung or kong.
#[inline]
pub(crate) fn is_pung_or_kong(m: &MeldInfo) -> bool {
    matches!(m.kind, MeldKind::Pung | MeldKind::Kong)
}

/// Check if a meld is a chow.
#[inline]
pub(crate) fn is_chow(m: &MeldInfo) -> bool {
    matches!(m.kind, MeldKind::Chow)
}

/// Check if all melds and the pair satisfy a rank range.
pub(crate) fn all_in_range(profile: &HandProfile, lo: u8, hi: u8) -> bool {
    let n = profile.n_sets as usize;
    profile.melds[..n].iter().all(|m| {
        let r = m.rank;
        lo <= r && r <= hi
    }) && {
        let r = profile.pair.rank;
        r > 0 && lo <= r && r <= hi
    }
}
