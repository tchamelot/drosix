import unittest
import control as ct
import numpy as np
import os
from utils import load

BASE_DIR = os.path.dirname(os.path.realpath(__file__))

SOURCES = [BASE_DIR + "/../src/pid.c"]
INCLUDE_PATH = [".", BASE_DIR + "/../src"]

module, ffi = load(SOURCES, INCLUDE_PATH, ["-std=c99"])
ffi.dlopen(None)

KP = 2541.0
TI = 1.75
TD = 41.52
N = 5
T = 0.01
MAX = 20000.0
MIN = -MAX


def gen_aw_pid(kp, ti, td, n, t, umax=1e10, kaw=0):
    def sat_update(t, x, u, params):
        return []

    def sat_output(t, x, u, params):
        return np.clip(u, -umax, umax)

    sat = ct.nlsys(
        sat_update, sat_output, name="sat", inputs=1, outputs=1, states=0, dt=t
    )

    proportional = ct.tf([kp], [1]).sample(t, method="bilinear", name="p")
    integrative = ct.tf([kp], [ti, 0]).sample(t, method="bilinear", name="i")
    derivative = ct.tf([kp * td, 0], [td / n, 1]).sample(t, method="bilinear", name="d")
    pid_sum = ct.tf([1], [1], name="sum", dt=t)
    aw_sum = ct.tf([1], [1, 0], name="aw", dt=t)
    return ct.interconnect(
        [proportional, integrative, derivative, pid_sum, sat, aw_sum],
        connections=[
            ["i.u", "aw.y"],
            ["aw.u", ("sum", "y", -kaw), ("sat", "y", kaw)],
            ["sum.u", "p.y", "i.y", "d.y"],
            ["sat.u", "sum.y"],
        ],
        inplist=[["p.u", "i.u", "d.u"]],
        outlist=[["sat.y"], ["sum.y"]],
        inputs=["error"],
        outputs=["y", "y_int"],
        name="pid",
        check_unused=False,
    )


def pid_setup(kp, ti, td, n, t, minimum=-1e10, maximum=1e10, kaw=0):
    pid = ffi.new("struct pid_controller_t*")
    pid_config = ffi.new("pid_config_t*")
    pid_config.kpr = kp
    pid_config.ti = ti
    pid_config.td = td
    pid_config.filter = n
    pid_config.max = maximum
    pid_config.min = minimum
    pid_config.kaw = kaw

    module.pid_init(pid, pid_config, t)

    return pid


class ProportionalOnly(unittest.TestCase):
    def setUp(self):
        self.pid = pid_setup(KP, 0, 0, 0, T)

    def test_coeff_p(self):
        np.testing.assert_allclose(self.pid.kd[0], KP, rtol=1e-5)
        np.testing.assert_allclose(self.pid.kd[1], 0, rtol=1e-5)
        np.testing.assert_allclose(self.pid.kd[2], 0, rtol=1e-5)
        np.testing.assert_allclose(self.pid.ki, 0, rtol=1e-5)

    def test_output(self):
        cmd = np.sin(np.arange(0, 50))
        out = [module.pid_run(self.pid, x) for x in cmd]
        np.testing.assert_allclose(out, KP * cmd, rtol=1e-5)


class SimplePid(unittest.TestCase):
    def setUp(self):
        self.pid = pid_setup(KP, TI, TD, N, T)
        self.i = ct.tf([KP], [TI, 0]).sample(T, method="bilinear")
        self.pd = KP * (1 + ct.tf([TD, 0], [TD / N, 1])).sample(T, method="bilinear")

    def test_coeff_i(self):
        np.testing.assert_allclose(self.pid.ki, self.i.num[0][0], rtol=1e-5)

    def test_coeff_pd(self):
        np.testing.assert_allclose(self.pid.kd[2], self.pd.den[0][0][1], rtol=1e-5)
        np.testing.assert_allclose(self.pid.kd[0], self.pd.num[0][0][0], rtol=1e-5)
        np.testing.assert_allclose(self.pid.kd[1], self.pd.num[0][0][1], rtol=1e-5)

    def test_output(self):
        time = np.arange(0, 50) * T
        ref_pid = self.i + self.pd
        out = [module.pid_run(self.pid, 1.0) for _ in time]
        ref = ct.step_response(ref_pid, time)
        np.testing.assert_allclose(out, ref.outputs, rtol=1e-5)


class SaturationPid(unittest.TestCase):
    def setUp(self):
        self.pid = pid_setup(KP, TI, TD, N, T, minimum=MIN, maximum=MAX)
        self.ref_pid = gen_aw_pid(KP, TI, TD, N, T, umax=MAX)
        self.time = np.arange(0, 100) * T

    def test_sat_max(self):
        step = 1.312
        ref = ct.input_output_response(self.ref_pid, T=self.time, U=step)
        out = [module.pid_run(self.pid, step) for _ in self.time]
        np.testing.assert_allclose(out, ref.outputs[0], rtol=1e-3)

    def test_sat_min(self):
        step = -1.312
        ref = ct.input_output_response(self.ref_pid, T=self.time, U=step)
        out = [module.pid_run(self.pid, step) for _ in self.time]
        np.testing.assert_allclose(out, ref.outputs[0], rtol=1e-3)

    def test_sat_both(self):
        step = np.sin(self.time * 2 * np.pi * 5) * 1.4
        ref = ct.input_output_response(self.ref_pid, T=self.time, U=step)
        out = [module.pid_run(self.pid, x) for x in step]
        np.testing.assert_allclose(out, ref.outputs[0], rtol=1e-3, atol=1)


class AntiWindupPid(unittest.TestCase):
    def setUp(self):
        self.pid = pid_setup(KP, TI, TD, N, T, minimum=MIN, maximum=MAX, kaw=1)
        self.ref_pid = gen_aw_pid(KP, TI, TD, N, T, umax=MAX, kaw=1)
        self.time = np.arange(0, 50) * T

    def test_output(self):
        step = np.sin(self.time * 2 * np.pi * 5) * 1.4
        ref = ct.input_output_response(self.ref_pid, T=self.time, U=step)
        out = [module.pid_run(self.pid, x) for x in step]
        np.testing.assert_allclose(out, ref.outputs[0], rtol=5e-2, atol=1)


if __name__ == "__main__":
    unittest.main()
