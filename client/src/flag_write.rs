fn lock_nightfarers() -> Option<String> {
    if NIGHTFARER_LOCKED.load(Ordering::Relaxed) {
        return None;
    }
    let keep = GRANTED_NIGHTFARERS.lock().unwrap().clone();
    let mut off = 0;
    let mut err = None;
    for flag in NIGHTFARER_FLAGS {
        if keep.contains(&flag) {
            continue;
        }
        match set_flag(flag, false) {
            Ok(_) => off += 1,
            Err(e) if err.is_none() => err = Some(e),
            Err(_) => {}
        }
    }
    if off == 0 {
        return Some(format!(
            "NRAP nightfarer clear waiting ({})",
            err.unwrap_or_else(|| "no flags".into())
        ));
    }
    NIGHTFARER_LOCKED.store(true, Ordering::Relaxed);
    Some(format!("NRAP nightfarer flags cleared {off}, kept {}", keep.len()))
}
