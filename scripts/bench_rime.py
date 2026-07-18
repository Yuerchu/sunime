"""
Benchmark RIME lookup latency for comparison with SunIME.

Usage: uv run python scripts/bench_rime.py
"""

import ctypes
import os
import time

WEASEL_DIR = r"C:\Program Files\Rime\weasel-0.17.4"
RIME_DLL = os.path.join(WEASEL_DIR, "rime.dll")
RIME_DIR = os.path.join(os.environ["APPDATA"], "Rime")

QUERIES = [
    "ni",
    "wo",
    "nihao",
    "zhongguo",
    "beijing",
    "zhuang",
    "zhonghuarenmingongheguo",
]

WARMUP = 3
ITERATIONS = 50


class RimeTraits(ctypes.Structure):
    _fields_ = [
        ("data_size", ctypes.c_int),
        ("shared_data_dir", ctypes.c_char_p),
        ("user_data_dir", ctypes.c_char_p),
        ("distribution_name", ctypes.c_char_p),
        ("distribution_code_name", ctypes.c_char_p),
        ("distribution_version", ctypes.c_char_p),
        ("app_name", ctypes.c_char_p),
        ("modules", ctypes.POINTER(ctypes.c_char_p)),
        ("min_log_level", ctypes.c_int),
        ("log_dir", ctypes.c_char_p),
        ("prebuilt_data_dir", ctypes.c_char_p),
        ("staging_dir", ctypes.c_char_p),
    ]


def main():
    rime = ctypes.CDLL(RIME_DLL)

    traits = RimeTraits()
    traits.data_size = ctypes.sizeof(RimeTraits)
    traits.shared_data_dir = os.path.join(WEASEL_DIR, "data").encode()
    traits.user_data_dir = RIME_DIR.encode()
    traits.distribution_name = b"Weasel"
    traits.distribution_code_name = b"Weasel"
    traits.distribution_version = b"0.17.4"
    traits.app_name = b"rime.bench"
    traits.min_log_level = 3

    print("Initializing RIME...")
    t0 = time.perf_counter()
    rime.RimeSetup(ctypes.byref(traits))
    rime.RimeInitialize(ctypes.byref(traits))

    rime.RimeStartMaintenance.restype = ctypes.c_int
    rime.RimeStartMaintenance(ctypes.c_int(0))
    rime.RimeJoinMaintenanceThread()
    init_time = time.perf_counter() - t0
    print(f"RIME initialized in {init_time:.3f}s")

    rime.RimeCreateSession.restype = ctypes.POINTER(ctypes.c_void_p)
    session_id = rime.RimeCreateSession()
    print(f"Session: {session_id}")

    rime.RimeSimulateKeySequence.restype = ctypes.c_int
    rime.RimeSimulateKeySequence.argtypes = [ctypes.c_void_p, ctypes.c_char_p]

    rime.RimeClearComposition.argtypes = [ctypes.c_void_p]

    print(f"\nBenchmark: {WARMUP} warmup + {ITERATIONS} iterations per query\n")
    print(f"{'Query':<35} {'Min':>10} {'Median':>10} {'P95':>10} {'P99':>10} {'Max':>10}")
    print("-" * 90)

    for query in QUERIES:
        times = []

        for i in range(WARMUP + ITERATIONS):
            rime.RimeClearComposition(session_id)

            t1 = time.perf_counter()
            rime.RimeSimulateKeySequence(session_id, query.encode())
            t2 = time.perf_counter()

            if i >= WARMUP:
                times.append((t2 - t1) * 1_000_000)

        times.sort()
        n = len(times)
        p50 = times[n // 2]
        p95 = times[int(n * 0.95)]
        p99 = times[int(n * 0.99)]

        print(
            f"{query:<35} {times[0]:>9.1f}us {p50:>9.1f}us {p95:>9.1f}us {p99:>9.1f}us {times[-1]:>9.1f}us"
        )

    rime.RimeDestroySession(session_id)
    rime.RimeFinalize()
    print("\nDone.")


if __name__ == "__main__":
    main()
