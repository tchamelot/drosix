import ctypes
from scipy.optimize import differential_evolution

rust = ctypes.CDLL('../target/release/libmodel_lib.so')

def wrapper(param, *args):
    return rust.pid_itae(
            ctypes.c_double(param[0]),
            ctypes.c_double(param[1]),
            ctypes.c_double(param[2]),
            ctypes.c_bool(False))

if __name__ == '__main__':
    rust.pid_itae.argtypes = [ctypes.c_double, ctypes.c_double, ctypes.c_double, ctypes.c_bool]
    rust.pid_itae.restype = ctypes.c_double
    res = differential_evolution(wrapper, [(0, 10), (30,50), (500, 1000)], workers=-1, polish=True)
    print('Tuned PID: ', res.x, ' itae = ', res.fun)
