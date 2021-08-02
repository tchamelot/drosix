import numpy as np
import control as ct
import matplotlib.pyplot as plt
import toml
import constants
from scipy.optimize import minimize, differential_evolution
from attitude import attitude
from motors import motors
import controller


def plot_model(model, t, u):
    print(model)
    t, y, x = ct.input_output_response(model, t, u, return_x=True)
    fig, ax = plt.subplots(2, 2)
    ax[0][0].plot(t, np.degrees(y[0]), label="x")
    ax[0][0].plot(t, np.degrees(y[1]), label="y")
    ax[0][0].plot(t, np.degrees(y[2]), label="z")
    ax[0][0].plot(t, np.degrees(u[0]))
    ax[0][0].legend()
    ax[0][0].set_title('angular position')
    ax[0][1].plot(t, np.degrees(y[3]), label="x")
    ax[0][1].plot(t, np.degrees(y[4]), label="y")
    ax[0][1].plot(t, np.degrees(y[5]), label="z")
    ax[0][1].plot(t, np.degrees(u[0]))
    ax[0][1].legend()
    ax[0][1].set_title('angular velocity')
    ax[1][0].plot(t, x[6] * constants.RADPS_2_RPM, label='motor 1')
    ax[1][0].plot(t, x[7] * constants.RADPS_2_RPM, label='motor 2')
    ax[1][0].plot(t, x[8] * constants.RADPS_2_RPM, label='motor 3')
    ax[1][0].plot(t, x[9] * constants.RADPS_2_RPM, label='motor 4')
    ax[1][0].plot(t, np.ones(t.shape) * 7500, 'r--')
    ax[1][0].plot(t, np.ones(t.shape) * -7500, 'r--')
    ax[1][0].legend()
    ax[1][0].set_title('motor velocity')
    ax[1][1].plot(t, y[-1])
    ax[1][1].set_title('thrust')
    plt.show()


def main():
    # Load model parameters
    drosix = toml.load('drosix_model.toml')

    """
    Build drone model
    """
    drone = ct.InterconnectedSystem(
        (attitude, motors),
        connections=(
            ('attitude.w1', 'motors.w1'),
            ('attitude.w2', 'motors.w2'),
            ('attitude.w3', 'motors.w3'),
            ('attitude.w4', 'motors.w4')),
        inplist=('motors.u1', 'motors.u2', 'motors.u3', 'motors.u4'),
        inputs=['u1', 'u2', 'u3', 'u4'],
        outlist=(
            'attitude.tx', 'attitude.ty', 'attitude.tz',
            'attitude.wx', 'attitude.wy', 'attitude.wz',
            'attitude.f'),
        outputs=['tx', 'ty', 'tz', 'wx', 'wy', 'wz', 'f'],
        name='drone',
        params=drosix)
    print(drone)
    
    """
    Linearize drone model at hover point
    """
    w_hover = drosix['hover']['w']
    throttle_hover = drosix['hover']['throttle']
    states_hover = [0, 0, 0, 0, 0, 0, w_hover, w_hover, w_hover, w_hover]
    inputs_hover = [throttle_hover, throttle_hover, throttle_hover, throttle_hover]
    drone_hover = ct.linearize(drone, states_hover, inputs_hover, params=drosix, copy=True, name='drone_hover')

    """
    Angular velocity PID tuning
    """
    samples = 1001
    t = np.linspace(0, 5, samples)
    wx = np.ones(t.shape) * 0.3
    wy = np.zeros(t.shape)
    x = np.ones(t.shape) * 0.17
    y = np.zeros(t.shape)
    # pid tuning
    # kph, tih, tdh = controller.tune_pid(
    #     drone_hover,
    #     (   ('drone_hover.u1', 'pid.y'),
    #         ('drone_hover.u2', '-pid.y'),
    #         ('drone_hover.u3', '-pid.y'),
    #         ('drone_hover.u4', 'pid.y'),
    #         ('pid.e', '-drone_hover.wx')),
    #         ('drone_hover.wx'),
    #         t,
    #         wx,
    #         [(0, 10), (30,50), (500, 1000)])
    kph, tih, tdh = [0.335, 40.60, 993.04]

    # kpz, tiz, tdz = pid_tune_z(drone_hover)

    pidx = controller.pid(kph, tih, tdh, name='pid_x')
    pidy = controller.pid(kph, tih, tdh, name='pid_y')
    # pidz = controller.pid(kpz, tiz, tdz, name='pid_z')

    drone_pid_w = ct.InterconnectedSystem(
        (drone_hover, pidx, pidy),
        connections=(
            ('drone_hover.u1', 'pid_x.y'),
            ('drone_hover.u1', 'pid_y.y'),
            # ('drone_hover.u1', 'pid_z.y'),
            ('drone_hover.u2', '-pid_x.y'),
            ('drone_hover.u2', 'pid_y.y'),
            # ('drone_hover.u2', '-pid_z.y'),
            ('drone_hover.u3', '-pid_x.y'),
            ('drone_hover.u3', '-pid_y.y'),
            # ('drone_hover.u3', 'pid_z.y'),
            ('drone_hover.u4', 'pid_x.y'),
            ('drone_hover.u4', '-pid_y.y'),
            # ('drone_hover.u4', '-pid_z.y'),
            ('pid_x.e', '-drone_hover.wx'),
            ('pid_y.e', '-drone_hover.wy')),
            # ('pid_z.e', '-drone_hover.wz')),
        inplist=('pid_x.e', 'pid_y.e'),
        inputs=['twx', 'twy'],
        outlist=(
            'drone_hover.tx', 'drone_hover.ty', 'drone_hover.tz',
            'drone_hover.wx', 'drone_hover.wy', 'drone_hover.wz',
            'drone_hover.f'),
        outputs=['tx', 'ty', 'tz', 'wx', 'wy', 'wz', 'f'],
        name='drone_pid_w')

    plot_model(drone_pid_w, t, np.array([wx, wy]))

    # kphp, tihp, tdhp = controller.tune_pid(
    #     drone_pid_w,
    #     (   ('drone_pid_w.twx', 'pid.y'),
    #         ('pid.e', '-drone_pid_w.tx')),
    #         ('drone_pid_w.tx'),
    #     t, x,
    #     [(1e-1, 1e1), (0, 0), (0, 0)])
    kphp, tihp, tdhp = [1.77, 0, 0]

    pidxp = controller.pid(kphp, tihp, tdhp, name='pid_xp')
    pidyp = controller.pid(kphp, tihp, tdhp, name='pid_yp')

    drone_pid_p = ct.InterconnectedSystem(
        (drone_pid_w, pidxp, pidyp),
        connections=(
            ('drone_pid_w.twx', 'pid_xp.y'),
            ('drone_pid_w.twy', 'pid_yp.y'),
            # ('drone_pid_w.tz', 'pid_zp.y'),
            ('pid_xp.e', '-drone_pid_w.tx'),
            ('pid_yp.e', '-drone_pid_w.ty')),
            #('pid_z.e', '-drone_pid_w.tz')),
        inplist=('pid_xp.e', 'pid_yp.e'),
        inputs=['twx', 'twy'],
        outlist=(
            'drone_pid_w.tx', 'drone_pid_w.ty', 'drone_pid_w.tz',
            'drone_pid_w.wx', 'drone_pid_w.wy', 'drone_pid_w.wz',
            'drone_pid_w.f'),
        outputs=['tx', 'ty', 'tz', 'wx', 'wy', 'wz', 'f'],
        name='drone_pid_p')

    plot_model(drone_pid_p, t, np.array([x, y]))


if __name__ == '__main__':
    main()
