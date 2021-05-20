import numpy as np
import control as ct


def attitude_update(t, x, u, params={}):
    """
    Drone dynamics for attitude control system.

    Parameters
    ----------
    x : array
        System state: drone angular position and velocity
    u : array
        System input: [speed; 4]  the speed of each propeller
        1   x   2    where
         \     /         propeller 1 and 3 spins anticlockwise
          --z-- y        propeller 2 and 4 spins clockwise
         /     \
        4       3

    Returns
    ----
    array [wx, wy, wz, ax, ay, az]
        Drone angular velocity
    """
    # Set up the system parameters
    d  = params['frame']['size'] * np.sqrt(2)   # Distance between drone center and propeller
    jx = params['frame']['jx']                  # Drone moment of inertia along the X axe
    jy = params['frame']['jy']                  # Drone moment of inertia along the Y axe
    jz = params['frame']['jz']                  # Drone moment of inertia along the Z axe
    Ct = params['propeller']['ct']              # Propeller thrust coefficient
    Cm = params['propeller']['cm']              # Propeller moment coefficient

    # A matrice
    wx = x[3]
    wy = x[4]
    wz = x[5]

    # TODO add disturbance

    # B matrice
    ax = d * Ct * (u[0] - u[1] - u[2] + u[3]) / jx
    ay = d * Ct * (u[0] + u[1] - u[2] - u[3]) / jy
    az = Cm * (u[0] - u[1] + u[2] - u[3]) / jz

    return [wx, wy, wz, ax, ay, az]


def attitude_output(t, x, u, params={}):
    Ct = params['propeller']['ct']              # Propeller thrust coefficient

    # compute thrust
    f = np.sum(u**2 * Ct)

    return np.append(x, f)


attitude = ct.NonlinearIOSystem(
    attitude_update, attitude_output, name='attitude',
    inputs=('w1', 'w2', 'w3', 'w4'),
    outputs=('tx', 'ty', 'tz', 'wx', 'wy', 'wz', 'f'),
    states=('tx', 'ty', 'tz', 'wx', 'wy', 'wz'),
    dt=0)
