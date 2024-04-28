import model
import pandas as pd
import matplotlib.pyplot as plt
from scipy.optimize import differential_evolution
import math


def plot_result(set_point):
    df = pd.read_csv("result.csv", header=None)

    plt.figure("Motor speed (rad/s)")
    plt.plot(df[0], df[1], "o--", label="motor 0")
    plt.plot(df[0], df[2], "o--", label="motor 1")
    plt.plot(df[0], df[3], "o--", label="motor 2")
    plt.plot(df[0], df[4], "o--", label="motor 3")
    plt.ticklabel_format(
        style="plain",
        useOffset=False,
    )
    plt.legend()

    plt.figure("Drone angular velocity (rad/s)")
    plt.plot(df[0], df[5], "o--", label="roll")
    plt.plot(df[0], df[12] - df[5], "o--", label="error")
    plt.plot(df[0], df[11], "o--", label="PID")
    # plt.plot(df[0], df[6], "o--", label="y")
    # plt.plot(df[0], df[7], "o--", label="z")
    plt.legend()

    plt.figure("Drone angular position (rad)")
    plt.plot(df[0], df[8], "o--", label="roll")
    plt.plot(df[0], set_point - df[8], "o--", label="error")
    plt.plot(df[0], df[12], "o--", label="PID")
    # plt.plot(df[0], df[9], "o--", label="y")
    # plt.plot(df[0], df[10], "o--", label="z")
    plt.legend()

    plt.show()


if __name__ == "__main__":
    set_point = math.radians(20)
    drosix = model.Model("drosix_model.toml", set_point)
    res = differential_evolution(
        drosix,
        [(1e3, 1e4), (1e3, 1e5), (1e3, 1e4), (1e0, 2e1)],
        # workers=-1,  # drosix callable object does not support multithreading
        updating="deferred",
        polish=True,
    )
    print(f"Tuned PID: {res.x[:-1]} {res.x[-1]}")
    vpid = model.Pid(res.x[0], res.x[1], res.x[2], 5, 0.01)
    ppid = model.Pid(res.x[3], 0, 0, 5, 0.01)

    # Current best: 0.004 4.0 95
    itae = drosix(res.x, save=True)
    # itae = drosix(np.array([0.004, 4.0, 95.0]), save=True)
    print(f"PID: {vpid} {ppid}, itae: {itae}")

    plot_result(set_point)
