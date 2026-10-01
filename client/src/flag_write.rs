/// Low unnamed flags. The setter kept 115 and did not keep 900203.
/// Everdark rows use their own flags so a base defeat does not open them.
pub fn flag_for_item(item_id: i64) -> Option<u32> {
    match item_id {
        839_100_002 => Some(190),
        839_100_003 => Some(191),
        839_100_004 => Some(192),
        839_100_005 => Some(193),
        839_100_006 => Some(194),
        839_100_007 => Some(195),
        839_100_008 => Some(115),
        839_100_009 => Some(135),
        839_100_010 => Some(136),
        839_100_111 => Some(210),
        839_100_112 => Some(211),
        839_100_113 => Some(212),
        839_100_114 => Some(213),
        839_100_115 => Some(214),
        839_100_116 => Some(215),
        839_100_117 => Some(216),
        839_100_118 => Some(217),
        _ => None,
    }
}
