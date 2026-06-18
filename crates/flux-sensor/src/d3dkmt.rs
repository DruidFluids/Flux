//! Live GPU metrics for non-NVIDIA (AMD / Intel, integrated + discrete) adapters
//! via the user-mode **D3DKMT** thunk interface (exports in `gdi32.dll`). This is
//! the same path Task Manager / LibreHardwareMonitor / HWiNFO use — no elevation,
//! no kernel driver, no vendor SDK. NVIDIA keeps using NVML; this fills the
//! clock / load / temperature that DXGI leaves as `None` for AMD & Intel.
#![cfg(windows)]
#![allow(dead_code)] // wired into read_gpu in a follow-up step

use std::time::Instant;
use windows::Wdk::Graphics::Direct3D::{
    D3DKMTCloseAdapter, D3DKMTOpenAdapterFromLuid, D3DKMTQueryAdapterInfo,
    D3DKMTQueryStatistics, D3DKMT_ADAPTER_PERFDATA, D3DKMT_CLOSEADAPTER,
    D3DKMT_NODE_PERFDATA, D3DKMT_OPENADAPTERFROMLUID, D3DKMT_QUERYADAPTERINFO,
    D3DKMT_QUERYSTATISTICS, D3DKMT_QUERYSTATISTICS_NODE, KMTQAITYPE_ADAPTERPERFDATA,
    KMTQAITYPE_NODEPERFDATA,
};
use windows::Win32::Foundation::{HANDLE, LUID};

/// Read `(clock_mhz, temp_c)` for the adapter with the given LUID. Either is
/// `None` when the driver doesn't populate it (returns 0 / a non-success status)
/// — many integrated GPUs report no separate temperature, which is expected.
pub fn read_clock_temp(luid: LUID) -> (Option<f32>, Option<f32>) {
    unsafe {
        let mut open = D3DKMT_OPENADAPTERFROMLUID { AdapterLuid: luid, hAdapter: 0 };
        if D3DKMTOpenAdapterFromLuid(&mut open).0 != 0 {
            return (None, None);
        }
        let hadapter = open.hAdapter;

        // Clock: node 0 = the primary 3D/graphics engine on consumer drivers.
        // NODE_PERFDATA.Frequency is in Hz.
        let mut clock_mhz = None;
        let mut perf = D3DKMT_NODE_PERFDATA::default();
        perf.NodeOrdinal = 0;
        perf.PhysicalAdapterIndex = 0;
        let mut qai = D3DKMT_QUERYADAPTERINFO {
            hAdapter: hadapter,
            Type: KMTQAITYPE_NODEPERFDATA,
            pPrivateDriverData: &mut perf as *mut _ as *mut core::ffi::c_void,
            PrivateDriverDataSize: core::mem::size_of::<D3DKMT_NODE_PERFDATA>() as u32,
        };
        if D3DKMTQueryAdapterInfo(&mut qai).0 == 0 && perf.Frequency > 0 {
            clock_mhz = Some(perf.Frequency as f32 / 1_000_000.0);
        }

        // Temperature: ADAPTER_PERFDATA.Temperature is in deci-Celsius (1 = 0.1°C).
        let mut temp_c = None;
        let mut ad = D3DKMT_ADAPTER_PERFDATA::default();
        ad.PhysicalAdapterIndex = 0;
        let mut qai2 = D3DKMT_QUERYADAPTERINFO {
            hAdapter: hadapter,
            Type: KMTQAITYPE_ADAPTERPERFDATA,
            pPrivateDriverData: &mut ad as *mut _ as *mut core::ffi::c_void,
            PrivateDriverDataSize: core::mem::size_of::<D3DKMT_ADAPTER_PERFDATA>() as u32,
        };
        if D3DKMTQueryAdapterInfo(&mut qai2).0 == 0 && ad.Temperature > 0 {
            temp_c = Some(ad.Temperature as f32 / 10.0);
        }

        let mut close = D3DKMT_CLOSEADAPTER { hAdapter: hadapter };
        let _ = D3DKMTCloseAdapter(&mut close);
        (clock_mhz, temp_c)
    }
}

/// Holds the previous utilization sample so usage % can be derived from the
/// delta of each engine's cumulative busy time between two polls.
#[derive(Default)]
pub struct UsageSampler {
    prev: Option<(Instant, Vec<i64>)>, // (sampled_at, running_time_100ns per node)
}

impl UsageSampler {
    /// Overall GPU usage % = the busiest engine's load (what Task Manager's
    /// headline GPU % shows), from the RunningTime delta vs the previous sample.
    /// `None` on the first call (no baseline) or when statistics are unavailable.
    pub fn read(&mut self, luid: LUID) -> Option<f32> {
        const MAX_NODES: u32 = 16;
        let now = Instant::now();
        let cur: Vec<i64> = (0..MAX_NODES)
            .map(|n| unsafe { query_node_running_time(luid, n) }.unwrap_or(0))
            .collect();

        let usage = match &self.prev {
            Some((t0, prev)) if prev.len() == cur.len() => {
                let elapsed_100ns = now.duration_since(*t0).as_nanos() as f64 / 100.0;
                if elapsed_100ns <= 0.0 {
                    None
                } else {
                    let max = cur
                        .iter()
                        .zip(prev)
                        .map(|(c, p)| ((c - p).max(0) as f64 / elapsed_100ns * 100.0).clamp(0.0, 100.0))
                        .fold(0.0f64, f64::max);
                    Some(max as f32)
                }
            }
            _ => None,
        };
        self.prev = Some((now, cur));
        usage
    }
}

/// Cumulative busy time (100ns ticks) of one engine/node, system-wide.
unsafe fn query_node_running_time(luid: LUID, node_id: u32) -> Option<i64> {
    let mut q: D3DKMT_QUERYSTATISTICS = core::mem::zeroed();
    q.Type = D3DKMT_QUERYSTATISTICS_NODE;
    q.AdapterLuid = luid;
    q.hProcess = HANDLE::default(); // NULL = system-wide (all processes)
    q.Anonymous.QueryNode.NodeId = node_id;
    if D3DKMTQueryStatistics(&q).0 != 0 {
        return None;
    }
    Some(q.QueryResult.NodeInformation.GlobalInformation.RunningTime)
}
