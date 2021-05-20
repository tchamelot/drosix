import numpy as np
import control as ct
from scipy.optimize import differential_evolution


def pid(kp, ti, td, n=5, name='pid'):
    """
    create a PID iosys with the given parameters
    """
    p = ct.tf([kp], [1])
    i = ct.tf([kp], [ti, 0]) if ti != 0 else ct.tf([0], [1])
    df = ct.tf([kp * td, 0], [td / n, 1]) if td != 0 else ct.tf([0], [1])
    return ct.tf2io(
        ct.parallel(p, i, df),
        inputs=['e'],
        outputs=['y'],
        name=name)


def itae_criterion(param, *args):
    """
    Compute the itae criterion for a PID with the given parameter
    param: [kp, ti, td]
    args: (model, connections, output, t, u) 
    """
    model, connections, output, t, u = args
    system = ct.InterconnectedSystem(
        (model, pid(param[0], param[1], param[2])),
        connections=connections,
        inplist=('pid.e'),
        outlist=(output))
    t, y, x = ct.input_output_response(system, t, u, return_x=True)
    itae = np.sum(np.abs(u -y) * t)
    if np.abs(np.max(x[6])) > 400:
        itae = np.inf
    return itae


def tune_pid(model, connections, output, t, u, bounds=[(1e-2, 1e1), (1e-2, 1e2), (1e1, 1e3)]):
    res = differential_evolution(
        itae_criterion, bounds,
        args=(model, connections, output, t, u),
        workers=-1,
        polish=False)
    print('Tuned PID: ', res.x, ' itae = ', res.fun)
    return res.x
