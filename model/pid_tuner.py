import model
import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
from scipy.optimize import differential_evolution

def plot_result():
    df = pd.read_csv("result.csv", header=None)

    plt.figure("Motor speed (rad/s)")
    plt.plot(df[0], df[1], "o--", label="motor 0")
    plt.plot(df[0], df[2], "o--", label="motor 1")
    plt.plot(df[0], df[3], "o--", label="motor 2")
    plt.plot(df[0], df[4], "o--", label="motor 3")
    plt.legend()

    plt.figure("Drone angular velocity (rad/s)")
    plt.plot(df[0], df[5], "o--", label="x")
    plt.plot(df[0], df[6], "o--", label="y")
    plt.plot(df[0], df[7], "o--", label="z")
    plt.legend()

    plt.figure("Drone angular position (rad)")
    plt.plot(df[0], df[8], "o--", label="x")
    plt.plot(df[0], df[9], "o--", label="y")
    plt.plot(df[0], df[10], "o--", label="z")
    plt.legend()

    plt.show()
    

if __name__ == '__main__':
    res = differential_evolution(model.pid_velocity_x, [(1000, 100000), (0, 10000), (0, 1)],
                                 workers=-1,
                                 updating='deferred',
                                 polish=True)
    print('Tuned PID: ', res.x, ' itae = ', res.fun)
    print(model.Pid(res.x[0], res.x[1], res.x[2], 5, 0.01))
    
    # model.pid_velocity_x(np.array([res.x[0], res.x[1], res.x[2]]), save=True)
    # plot_result()
