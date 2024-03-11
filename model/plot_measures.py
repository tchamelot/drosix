import struct
import matplotlib.pyplot as plt
import sys


def read_file(f):
    with open(f, "rb") as f:
        data = f.read()
        unpacked = []
        try:
            for i in range(0, len(data), 84):
                line = struct.unpack("<QQfffffffIIIIffffff", data[i : i + 84])
                unpacked.append(line)
        except struct.error:
            pass

        return unpacked


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print(f"usage: {sys.argv[0]} FILE")
        sys.exit(-1)
    data = list(zip(*read_file(sys.argv[1])))
    data = {
        "time": data[0],
        "roll": data[2],
        "pitch": data[3],
        "yaw": data[4],
        "vroll": data[6],
        "vpitch": data[7],
        "vyaw": data[8],
        "motor1": data[9],
        "motor2": data[10],
        "motor3": data[11],
        "motor4": data[12],
        "px": data[13],
        "py": data[14],
        "pz": data[15],
        "vx": data[16],
        "vy": data[17],
        "vz": data[18],
    }

    plt.figure("measure")
    plt.plot(data["time"], data["roll"], "ro--", label="roll")
    plt.plot(data["time"], data["pitch"], "go--", label="pitch")
    plt.plot(data["time"], data["yaw"], "bo--", label="yaw")
    plt.plot(data["time"], data["vroll"], "r+--", label="vroll")
    plt.plot(data["time"], data["vpitch"], "g+--", label="vpitch")
    plt.plot(data["time"], data["vyaw"], "b+--", label="vyaw")
    plt.legend()
    # plt.figure("motor")
    # plt.plot(data["time"], data["motor1"], "o--", label="1")
    # plt.plot(data["time"], data["motor2"], "o--", label="2")
    # plt.plot(data["time"], data["motor3"], "o--", label="3")
    # plt.plot(data["time"], data["motor4"], "o--", label="4")
    # plt.legend()
    # plt.figure("vpid")
    # plt.plot(data["time"], data["vx"], "o--", label="x")
    # plt.plot(data["time"], data["vy"], "o--", label="y")
    # plt.plot(data["time"], data["vz"], "o--", label="z")
    # plt.legend()
    # plt.figure("ppid")
    # plt.plot(data["time"], data["px"], "o--", label="x")
    # plt.plot(data["time"], data["py"], "o--", label="y")
    # plt.plot(data["time"], data["pz"], "o--", label="z")
    # plt.legend()
    plt.show()
