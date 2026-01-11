// Copyright 2025 Magic Mount-rs Authors
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod try_umount;

use std::sync::atomic::AtomicBool;

pub static KSU: AtomicBool = AtomicBool::new(false);

pub fn check_ksu() {
    let status = ksu::version().is_some_and(|v| {
        log::info!("KernelSU Version: {v}");
        true
    });

    KSU.store(status, std::sync::atomic::Ordering::Relaxed);
}
