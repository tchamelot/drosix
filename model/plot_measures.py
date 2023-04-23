import struct
import matplotlib.pyplot as plt


def read_file(f):
    with open(f, 'rb') as f:
        data = f.read()
        unpacked = []
        try:
            for i in range(0, len(data), 84):
                line = struct.unpack('<QQfffffffIIIIffffff', data[i:i+84])
                unpacked.append(line)
        except struct.error:
            pass

        return unpacked

if __name__ == '__main__':
    data = list(zip(*read_file('../measure')))
    data = {'time': data[0],
            'roll': data[2],
            'pitch': data[3],
            'yaw': data[4],
            'vroll': data[6],
            'vpitch': data[7],
            'vyaw': data[8],
            'motor1': data[9],
            'motor2': data[10],
            'motor3': data[11],
            'motor4': data[12],
            'vx': data[13],
            'vy': data[14],
            'vz': data[15],
            'px': data[16],
            'py': data[17],
            'pz': data[18]
            }

    plt.figure("measure")
    plt.plot(data['time'], data['roll'], 'o--', label='roll')
    plt.plot(data['time'], data['pitch'], 'o--', label='pitch')
    plt.plot(data['time'], data['yaw'], 'o--', label='yaw')
    plt.legend()
    plt.figure("vmeasure")
    plt.plot(data['time'], data['vroll'], 'o--', label='roll')
    plt.plot(data['time'], data['vpitch'], 'o--', label='pitch')
    plt.plot(data['time'], data['vyaw'], 'o--', label='yaw')
    plt.legend()
    plt.figure("motor")
    plt.plot(data['time'], data['motor1'], 'o--', label='1')
    plt.plot(data['time'], data['motor2'], 'o--', label='2')
    plt.plot(data['time'], data['motor3'], 'o--', label='3')
    plt.plot(data['time'], data['motor4'], 'o--', label='4')
    plt.legend()
    plt.figure('vpid')
    plt.plot(data['time'], data['vx'], 'o--', label='x')
    plt.plot(data['time'], data['vy'], 'o--', label='y')
    plt.plot(data['time'], data['vz'], 'o--', label='z')
    plt.legend()
    plt.figure('ppid')
    plt.plot(data['time'], data['px'], 'o--', label='x')
    plt.plot(data['time'], data['py'], 'o--', label='y')
    plt.plot(data['time'], data['pz'], 'o--', label='z')
    plt.legend()
    plt.show()


