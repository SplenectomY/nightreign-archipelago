                Ok(found) => {
                    log_line(&dir, &found.describe());
                    shop_mans = flagman::collect_all();
                    log_line(&dir, &format!("NRAP shopscan candidates={}", shop_mans.len()));
                    match flag_write::set_flag(6030, false) {
                        Ok(msg) => log_line(&dir, &format!("NRAP nightfarer probe {msg}")),
                        Err(e) => log_line(&dir, &format!("NRAP nightfarer probe failed: {e}")),
                    }
                    man = Some(found);
                }
