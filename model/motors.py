import numpy as np
import control as ct


def motors_update(t, x, u, params={}):
    """
    Motor dynamics for thrust control system

    Paramters
    ---------
    x: array
        System state: motors speed
    u: array
        System input: motors throttle (between 0 and 199999)

    Return
    ------
    array [a1, a2, a3, a4]
    """
    tm = params['motor']['tm']       # Motor torque constant
    cr = params['motor']['cr']       # Motor speed constant
    wb = params['motor']['wb']      # Motor base speed

    u = np.clip(u, 0, 1)
    accel = [(cr * throttle + wb - speed) / tm for throttle, speed in zip(u, x)]

    return accel


def motors_output(t, x, u, params={}):
    return x


motors = ct.NonlinearIOSystem(
    motors_update, motors_output, name='motors',
    inputs=('u1', 'u2', 'u3', 'u4'),
    outputs=('w1', 'w2', 'w3', 'w4'),
    states=('w1', 'w2', 'w3', 'w4'),
    dt=0)
