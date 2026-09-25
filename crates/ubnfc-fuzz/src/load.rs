//! ホスト負荷に応じて driver の壁時計タイムアウトを伸ばす（issue #73）。
//!
//! `/proc/loadavg` の 1 分平均をオンライン CPU 数で割った値を「1 CPU あたりの負荷」とし、
//! それを基準タイムアウトに掛ける倍率にする。無負荷（1 CPU あたり 1 未満）では倍率 1（=
//! 基準タイムアウトのまま、既定動作は変えない）。倍率は 4 を上限にクランプする
//! （上限を置かないと、恒常的に高負荷なホストでは driver が本当に停止していても
//! いつまでも待ち続けてしまう）。
//!
//! `/proc/loadavg` が無い環境（Linux 以外）では倍率 1 のまま動く。

use std::time::Duration;

/// 倍率の下限。基準タイムアウトより短くはしない。
const MIN_FACTOR: f64 = 1.0;
/// 倍率の上限。恒常的な高負荷でも driver 停止の検知が無限に遅れないための天井。
const MAX_FACTOR: f64 = 4.0;

/// `/proc/loadavg` の 1 分平均。読めない・パースできない環境では `None`。
fn load_average_1m() -> Option<f64> {
    let text = std::fs::read_to_string("/proc/loadavg").ok()?;
    text.split_whitespace().next()?.parse::<f64>().ok()
}

/// オンライン CPU 数の見積もり。取得できなければ 1（負荷率を大きめに見積もる安全側）。
fn online_cpus() -> f64 {
    std::thread::available_parallelism()
        .map(|n| n.get() as f64)
        .unwrap_or(1.0)
}

/// 1 CPU あたりの負荷から求めた倍率。`[MIN_FACTOR, MAX_FACTOR]` にクランプする。
fn clamp(per_cpu: f64) -> f64 {
    if per_cpu.is_finite() {
        per_cpu.clamp(MIN_FACTOR, MAX_FACTOR)
    } else {
        MIN_FACTOR
    }
}

/// 現在の負荷率から求めた倍率。`/proc/loadavg` が読めなければ 1 倍。
pub fn factor() -> f64 {
    let Some(load) = load_average_1m() else {
        return MIN_FACTOR;
    };
    clamp(load / online_cpus())
}

/// `base` に現在の負荷率を掛けたタイムアウト。無負荷なら `base` と等しい。
pub fn scale(base: Duration) -> Duration {
    base.mul_f64(factor())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_never_shrinks_below_one_and_caps_at_four() {
        assert_eq!(clamp(0.0), MIN_FACTOR);
        assert_eq!(clamp(0.3), MIN_FACTOR);
        assert_eq!(clamp(1.0), 1.0);
        assert_eq!(clamp(2.5), 2.5);
        assert_eq!(clamp(4.0), MAX_FACTOR);
        assert_eq!(clamp(1000.0), MAX_FACTOR);
    }

    #[test]
    fn clamp_treats_non_finite_input_as_unloaded() {
        assert_eq!(clamp(f64::NAN), MIN_FACTOR);
        assert_eq!(clamp(f64::INFINITY), MIN_FACTOR);
    }

    #[test]
    fn scale_is_identity_at_factor_one_and_linear_otherwise() {
        assert_eq!(
            Duration::from_secs(90).mul_f64(clamp(1.0)),
            Duration::from_secs(90)
        );
        assert_eq!(
            Duration::from_secs(90).mul_f64(clamp(2.0)),
            Duration::from_secs(180)
        );
    }

    /// このホストで実際に読める値であることの確認（CI コンテナでも `/proc/loadavg` は
    /// 通常存在する）。値そのものは環境依存なので範囲だけを検証する。
    #[test]
    fn factor_on_this_host_is_within_bounds() {
        let value = factor();
        assert!((MIN_FACTOR..=MAX_FACTOR).contains(&value), "{value}");
    }
}
