import matplotlib.pyplot as plt
import matplotlib.gridspec as gridspec
from matplotlib.animation import FuncAnimation
from matplotlib.widgets import CheckButtons
import numpy as np
from collections import deque
import socket
import struct
import threading
import queue
import datetime
import argparse


TO_DEG = 180 / np.pi


class DrosixSink:
    def __init__(self, address: str = "0.0.0.0", port: int = 9000, file=None):
        self.run = threading.Event()
        self.run.set()
        self.queue = queue.Queue()
        self.iter_buf_size = 3
        if file:
            self.thread = threading.Thread(target=self._file_receiver, args=(file,))
            self.iter_buf_size = 1000
        else:
            self.thread = threading.Thread(
                target=self._udp_receiver, args=(address, port)
            )
        self.thread.start()

    def stop(self):
        self.run.clear()
        self.thread.join()

    def _udp_receiver(self, address, port):
        now = datetime.datetime.now()
        save_name = f"logs/drosix_{now.year}-{now.month:02d}-{now.day:02d}-{now.hour:02d}-{now.minute:02d}.log"
        save_fd = open(save_name, "wb")
        self.socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self.socket.settimeout(1)
        self.socket.bind((address, port))
        while self.run.is_set():
            try:
                data = self.socket.recv(1024)
            except socket.timeout:
                continue
            save_fd.write(data)
            self._parse(data)

    def _file_receiver(self, path):
        fd = open(path, "rb")
        while self.run.is_set():
            data = fd.read()
            self._parse(data)

    def _parse(self, data):
        while len(data) > 12:
            if data[12:].startswith(b"MEASURE"):
                time = float(data[1:10])
                # command: thrust roll pitch yaw
                # sensor: roll pitch yaw droll dpitch dyaw thrust
                # pos pid: roll pitch yaw
                # vel pid: roll pitch yaw
                measures = list(struct.unpack("<ffff fffffff fff fff", data[20:88]))
                self.queue.put((time, measures))
                data = data[89:]
            else:
                line, data = data.split(b"\n", maxsplit=1)
                print(f"LOG: {line.decode()}")

    def __iter__(self):
        frame = np.empty((self.iter_buf_size, 18))
        cursor = 0
        flush = False
        while self.run.is_set():
            try:
                (time, measures) = self.queue.get(timeout=0.03)
                frame[cursor, 0] = time
                frame[cursor, 1:] = measures
                cursor += 1
            except queue.Empty:
                flush = True

            if cursor >= frame.shape[0] or flush:
                frame[:cursor, 1:4] *= 5
                frame[:cursor, 4:10] *= TO_DEG
                yield frame[:cursor].T
                cursor = 0
                flush = False


class Graph:
    def __init__(
        self,
        ax: plt.Axes,
        title: str,
        labels: [str],
        ylim: tuple[float, float],
        with_ref: bool = False,
    ):
        ax.set_title(title)
        ax.set_ylim(*ylim)
        ax.get_xaxis().set_visible(False)
        self.ax = ax
        self.lines = {}
        for label in labels:
            (self.lines[label],) = self.ax.plot([], [], label=label)

        rax = self.ax.inset_axes([1.0, 0.8, 0.15, 0.2])
        colors = [line.get_color() for line in self.lines.values()]
        self.check = CheckButtons(
            ax=rax,
            labels=labels,
            actives=[True] * len(labels),
            check_props={"facecolor": colors},
        )
        self.check.on_clicked(self.on_checkbox)
        self.previous_check = []
        if with_ref:
            self.refs = {}
            for label, color in zip(labels, colors):
                (self.refs[label],) = self.ax.plot([], [], color=color, linestyle=":")
        self.with_ref = with_ref

    def set_visible(self, visible: bool):
        self.ax.set_visible(visible)
        if visible:
            for i, state in enumerate(self.previous_check):
                self.check.set_active(i, state)
        else:
            self.previous_check = self.check.get_status()
            self.check.clear()

    def set_data(self, time: [float], data: [[float]], ref: [[float]] = []):
        for i, line in enumerate(self.lines.values()):
            line.set_data(time, data[i])
        if self.with_ref:
            for i, line in enumerate(self.refs.values()):
                line.set_data(time, ref[i])

        up = max(time)
        down = min(time)
        self.ax.set_xlim(down, up)

    def on_checkbox(self, label: str | None):
        if label:
            line = self.lines[label]
            line.set_visible(not line.get_visible())
            if self.with_ref:
                line = self.refs[label]
                line.set_visible(not line.get_visible())

        else:
            for line in self.lines.values():
                line.set_visible(False)
            if self.with_ref:
                for line in self.refs.values():
                    line.set_visible(False)
        line.figure.canvas.draw_idle()


# TODO add better UI: color / name / units
class Plotter:
    def __init__(self, sink, window_size=500):
        self.sink = sink
        self.fig = plt.figure()
        self.fig.subplots_adjust(hspace=0.3, wspace=0.3)
        self.gs_full = gridspec.GridSpec(2, 2)
        self.gs_single = gridspec.GridSpec(1, 1)
        self.zoomed = False
        self.time = deque(maxlen=window_size)
        self.ref = [
            deque(maxlen=window_size),
            deque(maxlen=window_size),
            deque(maxlen=window_size),
        ]
        self.pos = [
            deque(maxlen=window_size),
            deque(maxlen=window_size),
            deque(maxlen=window_size),
        ]
        self.vel = [
            deque(maxlen=window_size),
            deque(maxlen=window_size),
            deque(maxlen=window_size),
        ]
        self.p_pid = [
            deque(maxlen=window_size),
            deque(maxlen=window_size),
            deque(maxlen=window_size),
        ]
        self.v_pid = [
            deque(maxlen=window_size),
            deque(maxlen=window_size),
            deque(maxlen=window_size),
        ]

        self.ax_pos = self.fig.add_subplot(self.gs_full[0], label="Position")
        self.ax_vel = self.fig.add_subplot(
            self.gs_full[1], label="Velocity", sharex=self.ax_pos
        )
        self.ax_ppid = self.fig.add_subplot(
            self.gs_full[2], label="Position PID", sharex=self.ax_pos
        )
        self.ax_vpid = self.fig.add_subplot(
            self.gs_full[3], label="Velocity PID", sharex=self.ax_pos
        )

        self.graph = {}
        self.graph["Position"] = Graph(
            ax=self.ax_pos,
            title="Position",
            labels=["Roll", "Pitch", "Yaw"],
            ylim=(-45, 45),
            with_ref=True,
        )
        self.graph["Velocity"] = Graph(
            ax=self.ax_vel,
            title="Velocity",
            labels=["Roll", "Pitch", "Yaw"],
            ylim=(-50, 50),
        )
        self.graph["Position PID"] = Graph(
            ax=self.ax_ppid,
            title="Position PID",
            labels=["Roll", "Pitch", "Yaw"],
            ylim=(-10, 10),
        )
        self.graph["Velocity PID"] = Graph(
            ax=self.ax_vpid,
            title="Velocitiy PID",
            labels=["Roll", "Pitch", "Yaw"],
            ylim=(-50000, 50000),
        )

        self.fig.canvas.mpl_connect("button_press_event", self.on_click)
        self.fig.canvas.mpl_connect("key_press_event", self.on_key)

        self.anim = FuncAnimation(
            self.fig,
            self.animate,
            frames=self.sink,
            interval=25,
            cache_frame_data=False,
            repeat=False,
            # blit=True,
        )
        self.running = True

    def on_click(self, event):
        # prevent our handler when user use widget from toolbar such as zoom
        mode = plt.get_current_fig_manager().toolbar.mode
        ax_zoom = event.inaxes
        if not self.zoomed and mode == "" and ax_zoom is not None:
            for label, graph in self.graph.items():
                if label != ax_zoom._label:
                    graph.set_visible(False)
                else:
                    graph.set_visible(True)
                    graph.ax.set_position(
                        self.gs_single[0].get_position(event.canvas.figure)
                    )
            self.zoomed = True
            plt.draw()

    def on_key(self, event):
        if event.key == "escape" and self.zoomed:
            for i, (label, graph) in enumerate(self.graph.items()):
                graph.set_visible(True)
                graph.ax.set_position(self.gs_full[i].get_position(event.canvas.figure))
            self.zoomed = False
            plt.draw()
        elif event.key == "q":
            plt.close(event.canvas.figure)
            self.sink.stop()
        elif event.key == " ":
            if self.running:
                self.anim.pause()
                self.running = False
            else:
                self.anim.resume()
                self.running = True

    def animate(self, frame):
        if frame.size > 0:
            self.time.extend(frame[0])
            [self.ref[i].extend(frame[2 + i]) for i in range(0, 3)]
            [self.pos[i].extend(frame[5 + i]) for i in range(0, 3)]
            [self.vel[i].extend(frame[8 + i]) for i in range(0, 3)]
            [self.p_pid[i].extend(frame[12 + i]) for i in range(0, 3)]
            [self.v_pid[i].extend(frame[15 + i]) for i in range(0, 3)]

            self.graph["Position"].set_data(self.time, self.pos, ref=self.ref)
            self.graph["Velocity"].set_data(self.time, self.vel)
            self.graph["Position PID"].set_data(self.time, self.p_pid)
            self.graph["Velocity PID"].set_data(self.time, self.v_pid)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--udp", dest="port", default=9000)
    parser.add_argument("-f", "--file")
    args = parser.parse_args()
    print(args)

    if args.file:
        sink = DrosixSink(file=args.file)
        plotter = Plotter(sink, window_size=None)
    else:
        sink = DrosixSink(port=args.port)
        plotter = Plotter(sink)

    plt.show()
