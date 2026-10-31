//! What both viewers share for a measurement: the command line, the frame statistics and the result file.

use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Window size of every measurement, px.
pub const WINDOW: [u32; 2] = [1600, 900];
/// Frames left out at the start of a measurement (shader compilation, first uploads).
pub const WARM_UP_FRAMES: usize = 30;

/// The command line: `--bench SECONDS` measures and quits, `--glb FILE` draws pallets with a catalogue model,
/// `--out FILE` says where the result goes.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Args {
    pub bench: Option<f32>,
    pub glb: Option<PathBuf>,
    pub out: Option<PathBuf>,
}

impl Args {
    pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Args, String> {
        let mut parsed = Args::default();
        let mut it = args.into_iter().skip(1);
        while let Some(arg) = it.next() {
            let mut value = || it.next().ok_or_else(|| format!("{arg} needs a value"));
            match arg.as_str() {
                "--bench" => {
                    let secs: f32 = value()?.parse().map_err(|_| "--bench takes seconds".to_string())?;
                    if !(1.0..=600.0).contains(&secs) {
                        return Err("--bench takes 1 to 600 seconds".into());
                    }
                    parsed.bench = Some(secs);
                }
                "--glb" => parsed.glb = Some(PathBuf::from(value()?)),
                "--out" => parsed.out = Some(PathBuf::from(value()?)),
                other => return Err(format!("unknown argument {other}")),
            }
        }
        Ok(parsed)
    }

    /// The pallet model to use: the one given, else `pallet_a01.glb` next to the program, if there is one.
    pub fn glb_or_default(&self) -> Option<PathBuf> {
        if self.glb.is_some() {
            return self.glb.clone();
        }
        let beside = std::env::current_exe().ok()?.parent()?.join("pallet_a01.glb");
        beside.exists().then_some(beside)
    }
}

/// Frame-rate figures from the time each frame took.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FrameStats {
    pub frames: usize,
    pub seconds: f64,
    /// Frames per second over the whole measurement.
    pub mean_fps: f64,
    /// The rate of the slowest 5 % of frames ("1 in 20 frames is at least this slow").
    pub low_5pct_fps: f64,
    /// 95th percentile of the frame time, ms.
    pub p95_frame_ms: f64,
    /// Slowest frame, ms.
    pub max_frame_ms: f64,
}

impl FrameStats {
    pub fn from_frames(frames: &[Duration]) -> FrameStats {
        let frames = if frames.len() > WARM_UP_FRAMES * 2 { &frames[WARM_UP_FRAMES..] } else { frames };
        if frames.is_empty() {
            return FrameStats::default();
        }
        let mut ms: Vec<f64> = frames.iter().map(|d| d.as_secs_f64() * 1000.0).collect();
        let seconds: f64 = ms.iter().sum::<f64>() / 1000.0;
        ms.sort_by(f64::total_cmp);
        let p95 = ms[((ms.len() as f64 * 0.95).ceil() as usize).clamp(1, ms.len()) - 1];
        FrameStats {
            frames: ms.len(),
            seconds,
            mean_fps: if seconds > 0.0 { ms.len() as f64 / seconds } else { 0.0 },
            low_5pct_fps: if p95 > 0.0 { 1000.0 / p95 } else { 0.0 },
            p95_frame_ms: p95,
            max_frame_ms: ms[ms.len() - 1],
        }
    }
}

/// One viewer's result, written as JSON.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BenchResult {
    pub viewer: String,
    pub adapter: String,
    pub backend: String,
    pub window: [u32; 2],
    pub rollers: usize,
    pub pallets: usize,
    /// The pallet model drawn: a catalogue GLB, or boxes when there was none.
    pub pallet_model: String,
    /// Triangles of one pallet as drawn.
    pub pallet_triangles: usize,
    /// From the start of the program to the first frame on screen, ms.
    pub startup_ms: f64,
    /// Reading the pallet model, ms (0 for boxes).
    pub glb_load_ms: f64,
    pub frames: FrameStats,
    pub notes: Vec<String>,
}

impl BenchResult {
    /// Writes the result to `path`, or `result-<viewer>.json` beside the program, and prints it.
    pub fn write(&self, path: Option<&std::path::Path>) -> Result<PathBuf, String> {
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        let path = match path {
            Some(p) => p.to_path_buf(),
            None => std::env::current_exe()
                .ok()
                .and_then(|exe| exe.parent().map(|d| d.to_path_buf()))
                .unwrap_or_default()
                .join(format!("result-{}.json", self.viewer)),
        };
        std::fs::write(&path, json.as_bytes()).map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steady_frames_give_their_rate() {
        let frames = vec![Duration::from_micros(16_667); 600];
        let stats = FrameStats::from_frames(&frames);
        assert_eq!(stats.frames, 600 - WARM_UP_FRAMES);
        assert!((stats.mean_fps - 60.0).abs() < 0.1);
        assert!((stats.low_5pct_fps - 60.0).abs() < 0.1);
    }

    #[test]
    fn slow_frames_show_in_the_low_rate() {
        let mut frames = vec![Duration::from_millis(10); 1000];
        for f in frames.iter_mut().skip(WARM_UP_FRAMES).step_by(10) {
            *f = Duration::from_millis(50);
        }
        let stats = FrameStats::from_frames(&frames);
        assert!((stats.low_5pct_fps - 20.0).abs() < 0.1, "{stats:?}");
        assert!((stats.max_frame_ms - 50.0).abs() < 1e-9);
        assert!(stats.mean_fps < 100.0 && stats.mean_fps > 60.0);
    }

    #[test]
    fn no_frames_give_zeros() {
        assert_eq!(FrameStats::from_frames(&[]), FrameStats::default());
    }

    #[test]
    fn arguments_are_read_strictly() {
        let args = |v: &[&str]| Args::parse(std::iter::once("x".to_string()).chain(v.iter().map(|s| s.to_string())));
        assert_eq!(args(&["--bench", "30"]).map(|a| a.bench), Ok(Some(30.0)));
        assert!(args(&["--bench"]).is_err());
        assert!(args(&["--bench", "0"]).is_err());
        assert!(args(&["--bench", "abc"]).is_err());
        assert!(args(&["--fast"]).is_err());
        assert_eq!(args(&["--glb", "p.glb"]).map(|a| a.glb), Ok(Some(PathBuf::from("p.glb"))));
    }
}
