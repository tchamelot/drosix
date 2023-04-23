import model
import numpy as np
from scipy.optimize import differential_evolution



if __name__ == '__main__':
    res = differential_evolution(model.pid_velocity_x, [(1000, 100000), (0, 10000), (0, 1)],
                                 workers=-1,
                                 updating='deferred',
                                 polish=True)
    print('Tuned PID: ', res.x, ' itae = ', res.fun)
    print(model.Pid(res.x[0], res.x[1], res.x[2], 5, 0.01))
