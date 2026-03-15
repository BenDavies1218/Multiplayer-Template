//! Health thresholds for diagnostics metrics.
//!
//! Returns emoji status indicators (🟢/🟠/🔴) based on metric values.

/// Health status for a metric value.
#[derive(Clone, Copy)]
pub(crate) enum Health {
    Good,
    Warning,
    Critical,
}

impl Health {
    pub(crate) fn emoji(self) -> &'static str {
        match self {
            Health::Good => "🟢",
            Health::Warning => "🟠",
            Health::Critical => "🔴",
        }
    }
}

// ── Client thresholds ──

pub(crate) fn client_fps(fps: f64) -> Health {
    if fps > 55.0 {
        Health::Good
    } else if fps >= 30.0 {
        Health::Warning
    } else {
        Health::Critical
    }
}

pub(crate) fn frame_time_ms(ms: f64) -> Health {
    if ms < 18.0 {
        Health::Good
    } else if ms <= 33.0 {
        Health::Warning
    } else {
        Health::Critical
    }
}

pub(crate) fn rtt_ms(ms: f64) -> Health {
    if ms < 50.0 {
        Health::Good
    } else if ms <= 100.0 {
        Health::Warning
    } else {
        Health::Critical
    }
}

pub(crate) fn jitter_ms(ms: f64) -> Health {
    if ms < 10.0 {
        Health::Good
    } else if ms <= 30.0 {
        Health::Warning
    } else {
        Health::Critical
    }
}

pub(crate) fn rollbacks_per_sec(r: f64) -> Health {
    if r <= 5.0 {
        Health::Good
    } else if r <= 20.0 {
        Health::Warning
    } else {
        Health::Critical
    }
}

pub(crate) fn rollback_depth(d: f64) -> Health {
    if d <= 3.0 {
        Health::Good
    } else if d <= 10.0 {
        Health::Warning
    } else {
        Health::Critical
    }
}

// ── Server thresholds ──

pub(crate) fn server_fps(fps: f64) -> Health {
    if fps >= 60.0 {
        Health::Good
    } else if fps >= 30.0 {
        Health::Warning
    } else {
        Health::Critical
    }
}

pub(crate) fn tick_rate_pct(pct: f64) -> Health {
    if pct >= 95.0 {
        Health::Good
    } else if pct >= 80.0 {
        Health::Warning
    } else {
        Health::Critical
    }
}
