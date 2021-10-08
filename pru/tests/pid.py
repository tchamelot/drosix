import unittest
import control as ct
import numpy as np
import os
from utils import load

BASE_DIR = os.path.dirname(os.path.realpath(__file__))

SOURCES = [BASE_DIR + '/../src/pid.c']
INCLUDE_PATH = ['.', BASE_DIR + '/../src']

module, ffi = load(SOURCES, INCLUDE_PATH, ['-std=c99'])
ffi.dlopen(None)


KP = 10.0
TI = 1.0
TD = 1.0
T = 0.01


class TestPid(unittest.TestCase):
    def setUp(self):
        self.t = 0.0 + np.arange(0, 10) * T
        self.ct_pid = (KP + ct.tf([KP/TI], [1, 0]) + ct.tf([KP*TD, 0], [TD/5, 1])).sample(T, 'bilinear')
        self.ffi_pid = ffi.new('struct pid_t*')
        module.pid_init(self.ffi_pid,
                self.ct_pid.num[0][0].tolist(),
                self.ct_pid.den[0][0][1:].tolist())
    
    def testPid(self):
        _, ct_y = ct.step_response(self.ct_pid, self.t)
        ffi_y = [module.pid_run(self.ffi_pid, 1.0) for _ in self.t]
        np.testing.assert_allclose(ffi_y, ct_y, rtol=1e-5)


if __name__ == '__main__':
  unittest.main()
